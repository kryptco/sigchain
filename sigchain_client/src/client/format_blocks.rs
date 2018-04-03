/// Turn sigchain operations into human-readable text.

use serde_json;

use {Result, Client, SignedMessage, Message};

use b64data;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormattedBlock {
    header: String,
    author: String,
    time: String,
    body: Option<String>,
    #[serde(with = "b64data")]
    hash: Vec<u8>,
    first: bool,
    last: bool,
}

fn short_time_format(seconds: i64) -> String{
    use chrono::Duration;
    use std::ops::Sub;
    let duration = Duration::seconds(seconds);

    if duration.num_minutes() < 1 {
        return format!("{}s", duration.num_seconds());
    } else if duration.num_hours() < 1 {
        return format!("{}m", duration.num_minutes());
    } else if duration.num_days() < 1 {
        let remainder = duration.sub(Duration::hours(duration.num_hours())).num_minutes();
        return format!("{}h {}m", duration.num_hours(), remainder);
    } else {
        return format!("{} days", duration.num_days());
    }
}

pub fn format<C: Client>(c: &C, block: &SignedMessage) -> Result<FormattedBlock> {
    let msg: Message = serde_json::from_str(&block.message)?;

    use protocol::Body;
    let main_chain = match msg.body {
        Body::Main(main_chain) => {
            main_chain
        },
        _ => {
            bail!("formatting only supported for main chain blocks")
        },
    };

    use db;
    use db::TeamDBConnection;
    let conn = &TeamDBConnection{team: c.team_pk(), conn: c.db_conn()};

    let author = db::Identity::find(conn, &block.public_key)?;

    use MainChain::*;
    use team::Operation::*;
    let (header, body) = match main_chain.clone() {
        Append(append) => {
            match append.operation {
                Invite(invite) => {
                    use team::Invitation::*;
                    match invite {
                        Indirect(indirect) => {
                            use team::IndirectInvitationRestriction::*;
                            match indirect.restriction {
                                Domain(domain) => ("invite link", Some(format!("for @{} only", domain))),
                                Emails(emails) => ("invite link", Some(format!("for {}", emails.join(", ")))),
                            }
                        }
                        Direct(direct) => ("direct invitation", Some(format!("for {}", direct.email))),
                    }
                }
                CloseInvitations(_) => ("close invitations", None),
                AcceptInvite(identity) => ("accept invite", Some(format!("{} joined the team", identity.email))),
                Remove(public_key) => (
                    "remove",
                    Some(format!("remove {} from the team",
                                 db::Identity::find(conn, &public_key)?.email,
                    ),
                )),
                Leave(_) => ("leave team", None),
                SetPolicy(policy) => ("set policy",
                                      Some(format!("temporary approval {}",
                                                   match policy.temporary_approval_seconds {
                                                       Some(seconds) => short_time_format(seconds),
                                                       None => "unset".to_string(),
                                                   })
                                      )),
                SetTeamInfo(team_info) => ("set team name", Some(team_info.name)),
                PinHostKey(host_key) => ("pinned host", Some(host_key.host)),
                UnpinHostKey(host_key) => ("unpinned host", Some(host_key.host)),
                Promote(public_key) => ("promote", Some(format!("promote {} to admin",
                                                                db::Identity::find(conn, &public_key)?.email,
                ))),
                Demote(public_key) => ("demote", Some(format!("demote {} to member",
                                                                db::Identity::find(conn, &public_key)?.email,
                ))),
                AddLoggingEndpoint(_) => ("enable audit logging", None),
                RemoveLoggingEndpoint(_) => ("disable audit logging", None),
            }
        }
        Create(genesis_block) => ("create chain", Some(
            format!("team \"{}\" created",
                    genesis_block.team_info.name,
            )
        )),
        Read(_) => {
            bail!("formatting not supported for reads")
        }
    };

    use time_util::TimeAgo;

    Ok(FormattedBlock {
        header: header.into(),
        author: author.email,
        time: msg.header.utc_time.full_timestamp().trim().to_string(),
        body: body.map(String::from),
        hash: block.payload_hash(),
        first: main_chain.last_block_hash().is_none(),
        last: c.get_last_block_hash()?.map(|lbh| lbh == block.payload_hash()).unwrap_or(false),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FormattedRequestableOperation {
    pub header: String,
    pub body: String,
}
use enclave_protocol;
pub fn format_requestable_op<C: Client>(c: &C, op: enclave_protocol::RequestableTeamOperation) -> Result<FormattedRequestableOperation> {
    use base64;
    use db;
    use ssh;

    let conn = &db::TeamDBConnection{team: c.team_pk(), conn: c.db_conn()};

    use enclave_protocol::RequestableTeamOperation::*;
    let (header, body) = match op {
        AddLoggingEndpoint(_) => {
            ("Enable Logging", format!("Enable storing encrypted logs on the team server"))
        }
        RemoveLoggingEndpoint(_) => {
            ("Disable Logging", format!("Stop storing encrypted logs on the team server"))
        }

        DirectInvite(invite) => {
            ("In-person Invitation", format!("Invite {} with public key {} to the team", invite.email, base64::encode(&invite.public_key)))
        }
        IndirectInvite(restriction) => {
            use team::IndirectInvitationRestriction::*;
            match restriction {
                Domain(domain) => ("Create Invitation Link", format!("Create invite for @{} only emails", domain)),
                Emails(emails) => ("Create Invitation Link", format!("Create invite link for {}", emails.join(", "))),
            }
        }
        CloseInvitations(_) => {
            ("Close Invitations", format!("Close all open invitations to the team"))
        }

        SetPolicy(policy) => {
            ("Set Policy",
             match policy.temporary_approval_seconds {
                 Some(seconds) => format!("Set temporary approval duration to {}", short_time_format(seconds)),
                 None => format!("Un-set temporary approval duration"),
             })
        }
        SetTeamInfo(info) => {
            ("Set Team Name", format!("Set team name to {}", info.name))
        }

        PinHostKey(host) => ("Add Pinned Host Key", format!("Update {}'s pinned SSH public keys to include {}", host.host, ssh::ssh_public_key_wire_string(&host.public_key)?)),
        UnpinHostKey(host) => ("Remove Pinned Host Key", format!("Unpin {}'s SSH public key: {}", host.host, ssh::ssh_public_key_wire_string(&host.public_key)?)),

        Promote(pk) => ("Promote to Admin", format!("Promote {} to admin", db::Identity::find(conn, &pk)?.email)),
        Demote(pk) => ("Demote to Member", format!("Demote {} to member", db::Identity::find(conn, &pk)?.email)),
        Remove(pk) => ("Remove from Team", format!("Remove {} from the team", db::Identity::find(conn, &pk)?.email)),
        Leave(_) => ("Leave Team", format!("Leave the team")),
    };

    Ok(FormattedRequestableOperation{
        header: header.into(),
        body: body.into(),
    })
}
