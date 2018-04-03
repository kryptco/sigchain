pub use sigchain_core::diesel::prelude::*;
use serde_json;
use sigchain_core::errors;
use self::errors::{Result};
use protocol::*;
use db::{TeamDBConnection, DBConnection, uniqueness_to};
use crypto::ed25519;
use time;

use db;

#[allow(unused_imports)]
use std::borrow::Borrow;

use notification::*;

pub fn verify_and_process_request(conn: &DBConnection, request: &SignedMessage) -> Result<NotificationsAndResponse> {
    let verified_payload = &verify_signature_and_version(request)?;
    verify_and_process_request_payload(conn, request, verified_payload)
}

pub fn verify_and_process_request_payload(conn: &DBConnection, request: &SignedMessage, verified_payload: &Message) -> Result<NotificationsAndResponse> {
    match verified_payload.body.clone() {
        Body::Main(main_chain) => {
            use MainChain::*;
            match &main_chain {
                &Read(_) => {
                    bail!("unexpected ReadBlock")
                }
                &Append(ref write_block) => {
                    return append_block(conn, request, &main_chain, write_block);
                }
                &Create(ref create_chain) => {
                    if db::Block::exists(conn, &request.payload_hash())? {
                        bail!(errors::BlockExists);
                    }
                    return create_team(conn, request, &main_chain, create_chain)
                        .map(NotificationsAndResponse::no_notifications);
                }
            }
        },
        Body::Log(log_chain) => {
            use LogChain::*;
            match &log_chain {
                &Create(ref log_chain_genesis_block) => {
                    return create_log_chain(conn, request, &log_chain, log_chain_genesis_block)
                        .map(NotificationsAndResponse::no_notifications);
                }
                &Append(ref log_op) => {
                    return append_log(conn, request, &log_chain, log_op)
                        .map(NotificationsAndResponse::no_notifications);
                }
                &Read(_) => {
                    bail!("unexpected ReadLogBlock")
                }
            }
        },
        Body::ReadToken(_) => {
            bail!("send ReadToken with ReadBlocksRequest or ReadLogBlocksRequest")
        },
        Body::EmailChallenge(_) => {
            bail!("unexpected EmailChallenge")
        },
        Body::PushSubscription(_) => {
            bail!("unexpected PushSubscription")
        },
        Body::ReadBillingInfo(_) => {
            bail!("unexpected ReadBillingInfo")
        }
    }

}

pub fn team_pointer_to_public_key(conn: &DBConnection, team_pointer: &TeamPointer) -> Result<Vec<u8>> {
    use TeamPointer::*;
    Ok(
        match team_pointer {
            &PublicKey(ref team_public_key) => {
                team_public_key.clone()
            }
            &LastBlockHash(ref last_block_hash) => {
                let existing_block = db::Block::find(conn, last_block_hash)?;
                existing_block.team_public_key
            }
        }
    )
}

pub fn create_team(conn: &DBConnection, request: &SignedMessage, verified_payload: &MainChain, create_chain: &GenesisBlock) -> Result<String> {
    if *request.public_key != *create_chain.creator_identity.public_key {
        bail!(errors::NotAnAdmin)
    }

    let team_public_key = create_chain.creator_identity.public_key.clone();
    db::TeamMembership{
        team_public_key: team_public_key.clone(),
        member_public_key: team_public_key.clone(),
        email: create_chain.creator_identity.email.clone(),
        is_admin: true,
    }.insert(conn)?;

    db::Team{
        public_key: team_public_key.clone(),
        last_block_hash: request.payload_hash(),
        name: create_chain.team_info.name.clone(),
        temporary_approval_seconds: None,
        last_read_log_chain_logical_timestamp: None,
        command_encrypted_logging_enabled: false,
    }.insert(conn)?;

    db::Identity::from_identity(
        team_public_key.clone(), create_chain.creator_identity.clone()
    ).insert(conn)?;

    db::Block::build(
        request,
        verified_payload,
        team_public_key.clone(),
    )?.insert(conn)?;

    success!(E{})
}

pub fn append_block(conn: &DBConnection, request: &SignedMessage, verified_payload: &MainChain, write_block: &Block) -> Result<NotificationsAndResponse> {
    let team_public_key = db::Block::find(conn, &write_block.last_block_hash)?
        .team_public_key;
    let conn = &db::TeamDBConnection{conn, team: &team_public_key};
    let mut notification_actions = Vec::new();

    match write_block.operation {
        Operation::AcceptInvite(ref identity) => {
            if let Some(indirect_invite) = db::IndirectInvitation::find(conn, &request.public_key).optional()? {
                let restriction : IndirectInvitationRestriction = serde_json::from_str(&indirect_invite.restriction_json)?;
                match restriction {
                    IndirectInvitationRestriction::Domain(domain) => {
                        if !identity.email.ends_with(&("@".to_string() + &domain)) {
                            bail!(errors::InviteNotValid);
                        }
                    },
                    IndirectInvitationRestriction::Emails(emails) => {
                        if !emails.contains(&identity.email) {
                            bail!(errors::InviteNotValid);
                        }
                    },
                }
            } else if let Some(direct_invite) = db::DirectInvitation::find(conn, &request.public_key).optional()? {
                if !(direct_invite.public_key == request.public_key && request.public_key == identity.public_key) {
                    bail!(errors::InviteNotValid);
                }
                if !(direct_invite.email == identity.email) {
                    bail!(errors::InviteNotValid);
                }
            } else {
                bail!(errors::InviteNotValid);
            }
            if let Some(existing_direct_invite) = db::DirectInvitation::find(conn, &identity.public_key).optional()? {
                existing_direct_invite.delete(conn)?;
            }
        }
        Operation::Leave(_) => {
            //  Any member can leave the team
            db::TeamMembership::find(conn, &request.public_key)?;
        }
        _ => {
            if !db::TeamMembership::find(conn, &request.public_key)?.is_admin {
                bail!(errors::NotAnAdmin)
            }
        }
    }

    let block = db::Block::build(request, verified_payload, team_public_key.clone())?;

    let _last_block = db::Block::find(conn.conn, &write_block.last_block_hash)?;

    if db::Block::exists(conn.conn, &request.payload_hash())? {
        bail!(errors::BlockExists);
    }

    block.insert(conn.conn).map_err(|e| uniqueness_to(e, errors::NotAppendingToMainChain))?;
    db::Team::update_last_block_hash(conn, &block.hash)?;

    use Operation::*;
    match &write_block.operation {
        &Invite(ref invitation) => {
            use Invitation::*;
            match invitation {
                &Indirect(ref indirect_invitation) => {
                    if db::DirectInvitation::exists(conn, &indirect_invitation.nonce_public_key)? {
                        bail!("invitation key already in use");
                    }
                    if let IndirectInvitationRestriction::Emails(ref emails) = indirect_invitation.restriction {
                        for email in emails {
                            if db::TeamMembership::find_email(conn, &email).optional()?.is_some() {
                                bail!(errors::AlreadyOnTeam);
                            }
                        }
                    }
                    db::IndirectInvitation::from_invitation(
                        &team_public_key,
                        indirect_invitation.clone(),
                    )?.insert(conn)?;
                }
                &Direct(ref direct_invitation) => {
                    if db::IndirectInvitation::exists(conn, &direct_invitation.public_key)? {
                        bail!("invitation key already in use");
                    }
                    if db::TeamMembership::find(conn, &direct_invitation.public_key).optional()?.is_some() {
                        bail!(errors::AlreadyOnTeam)
                    }
                    if db::TeamMembership::find_email(conn, &direct_invitation.email).optional()?.is_some() {
                        bail!(errors::EmailInUse)
                    }
                    db::DirectInvitation::from_invitation(
                        &team_public_key,
                        direct_invitation.clone(),
                    ).insert(conn).map_err(|e| uniqueness_to(e, errors::CloseInvitationsFirst))?;
                }
            }
        }
        &CloseInvitations(_) => {
            db::IndirectInvitation::delete_team_invites(conn)?;
            db::DirectInvitation::delete_team_invites(conn)?;
        }
        &AcceptInvite(ref identity) => {
            if db::TeamMembership::find_email(conn, &identity.email).optional()?.is_some() {
                bail!(errors::EmailInUse{})
            }

            db::TeamMembership{
                team_public_key: team_public_key.clone(),
                member_public_key: identity.public_key.clone(),
                email: identity.email.clone(),
                is_admin: false,
            }.insert(conn.conn)?;

            db::Identity::from_identity(team_public_key.clone(), identity.clone())
                .insert_or_update(conn.conn)?;
        }
        &Remove(ref public_key) => {
            if *public_key == request.public_key {
                bail!("cannot remove self, use Leave op instead")
            }
            let removed_membership = db::TeamMembership::find(conn, &public_key)?;
            removed_membership.delete(conn.conn)?;

            notification_actions.push(NotificationAction::Unsubscribe(removed_membership.clone()));

            //  Close all invites
            db::IndirectInvitation::delete_team_invites(conn)?;
            db::DirectInvitation::delete_team_invites(conn)?;
        }
        &Leave(_) => {
            let removed_membership = db::TeamMembership::find(conn, &request.public_key)?;
            removed_membership.delete(conn.conn)?;

            notification_actions.push(NotificationAction::Unsubscribe(removed_membership.clone()));
        }
        &Promote(ref public_key) => {
            let mut membership = db::TeamMembership::find(conn, &public_key)?;
            if membership.is_admin {
                bail!("already an admin");
            }
            membership.is_admin = true;
            membership.update(conn.conn)?;
        }
        &Demote(ref public_key) => {
            let mut membership = db::TeamMembership::find(conn, &public_key)?;
            if !membership.is_admin {
                bail!("not an admin");
            }
            membership.is_admin = false;
            membership.update(conn.conn)?;
        }
        &SetPolicy(ref policy) => {
            let mut team = db::Team::find(conn)?;
            team.temporary_approval_seconds = policy.temporary_approval_seconds;
            team.update(conn.conn)?;
        }
        &SetTeamInfo(ref team_info) => {
            let mut team = db::Team::find(conn)?;
            team.name = team_info.name.clone();
            team.update(conn.conn)?;
        }
        &PinHostKey(ref host_key) => {
            db::PinnedHostKey {
                team_public_key: team_public_key.clone(),
                host: host_key.host.clone(),
                public_key: host_key.public_key.clone(),
            }.insert(conn.conn)?;
        }
        &UnpinHostKey(ref host_key) => {
            let db_pinned_key = db::PinnedHostKey {
                team_public_key: team_public_key.clone(),
                host: host_key.host.clone(),
                public_key: host_key.public_key.clone(),
            };
            if !db_pinned_key.exists(conn.conn)? {
                bail!("host key not pinned")
            }
            db_pinned_key.delete(conn.conn)?;
        }
        &AddLoggingEndpoint(ref logging_endpoint) => {
            match logging_endpoint {
                &LoggingEndpoint::CommandEncrypted(_) => {
                    let mut team = db::Team::find(conn)?;
                    if team.command_encrypted_logging_enabled {
                        bail!("logging already enabled");
                    }
                    team.command_encrypted_logging_enabled = true;
                    team.update(conn.conn)?;
                }
            }
        }
        &RemoveLoggingEndpoint(ref logging_endpoint) => {
            match logging_endpoint {
                &LoggingEndpoint::CommandEncrypted(_) => {
                    let mut team = db::Team::find(conn)?;
                    if !team.command_encrypted_logging_enabled {
                        bail!("logging not enabled");
                    }
                    team.command_encrypted_logging_enabled = false;
                    team.update(conn.conn)?;
                }
            }
        }
    };

    notification_actions.push(NotificationAction::TeamPush(team_public_key.clone()));
    Ok(NotificationsAndResponse {
        json_response_to_client: success_string!(E{}),
        notification_actions,
    })
}

pub fn create_log_chain(conn: &DBConnection, request: &SignedMessage, verified_payload: &LogChain, create_log_chain: &GenesisLogBlock) -> Result<String> {
    if db::LogBlock::exists(conn, &request.payload_hash())? {
        bail!(errors::BlockExists);
    }
    let team_public_key = team_pointer_to_public_key(conn, &create_log_chain.team_pointer)?;
    let conn = &db::TeamDBConnection{conn, team: &team_public_key};

    //  Allow removed members' blocks to still be processed so that admins can read a LogChain of a since-removed member
    db::Identity::find(conn, &request.public_key)?;

    let block = db::LogBlock::build(
        request,
        verified_payload,
        team_public_key.clone(),
    )?;
    block.insert(conn.conn)?;

    db::LogChain{
        team_public_key: team_public_key.clone(),
        member_public_key: request.public_key.clone(),
        last_block_hash: block.hash.clone(),
        symmetric_encryption_key: None,
    }.insert(conn.conn)?;

    success!(E{})
}

pub fn append_log(conn: &DBConnection, request: &SignedMessage, verified_payload: &LogChain, append_log: &LogBlock) -> Result<String> {
    if db::LogBlock::exists(conn, &request.payload_hash())? {
        bail!(errors::BlockExists);
    }

    let last_block = db::LogBlock::find_for_member(conn, &request.public_key, &append_log.last_block_hash)?;

    let conn = &TeamDBConnection{conn, team: &last_block.team_public_key};

    if db::LogBlock::next_block_exists(conn, &request.public_key, &Some(last_block.hash))? {
        bail!(errors::NotAppendingToMainChain{})
    }

    db::LogBlock::build(
        request,
        verified_payload,
        conn.team.into(),
    )?.insert(conn.conn)?;

    db::LogChain::update_last_block_hash(conn, &request.public_key, request.payload_hash())?;

    success!(E{})
}



// verification helper functions

/// Perform verifications local to the payload data
/// Does NOT verify team membership or block chain existence/length/structure
pub fn verify_signature_and_version(request: &SignedMessage) -> Result<Message> {
    let sig = match ed25519::Signature::from_slice(request.signature.as_ref()) {
        Some(sig) => sig,
        None => bail!("invalid signature"),
    };

    let pk = match ed25519::PublicKey::from_slice(&request.public_key) {
        Some(pk) => pk,
        None => bail!("invalid public key"),
    };

    if !ed25519::verify_detached(&sig, request.message.as_bytes(), &pk) {
        bail!("signature verification failed");
    }

    let payload : Message = serde_json::from_str(&request.message)?;

    if payload.header.protocol_version.major > CURRENT_VERSION.major {
        bail!(errors::VersionIncompatible)
    }

    Ok(payload)
}

pub fn map_read_token_to_identity_pk(request_public_key: &[u8], signed_token: &Option<SignedReadToken>) -> Result<Vec<u8>> {
    let signed_token = match signed_token {
        &Some(ref token) => token,
        &None => return Ok(request_public_key.into()),
    };

    let token = verify_signature_and_version(signed_token)?;
    match token.body {
        Body::ReadToken(ReadToken::Time(time_token)) => {
            if *request_public_key != *time_token.reader_public_key {
                bail!("ReadToken does not match request")
            }
            if time::get_time().sec > time_token.expiration {
                bail!("read token expired")
            }
            Ok(signed_token.public_key.clone())
        },
        _ => {
            bail!("not a ReadToken")
        }
    }
}