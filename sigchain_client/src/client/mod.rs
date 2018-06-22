extern crate serde_json;
extern crate time;

pub use sigchain_core::diesel::prelude::*;

#[cfg(feature = "network_client")]
use {enclave_protocol, crypto};

use {team, protocol, DBConnection, db};

#[cfg(feature = "network_client")]
use krd_client;

use std;

use errors::{Result, Error};
use super::crypto::*;
use serde;

#[cfg(feature = "network_client")]
mod network_client;
#[cfg(feature = "network_client")]
pub use self::network_client::*;

#[cfg(feature = "network_client")]
mod delegated_client;
#[cfg(feature = "network_client")]
pub use self::delegated_client::*;

#[cfg(target_os = "android")]
pub mod android_client;
#[cfg(target_os = "android")]
pub use self::android_client::*;

mod test_client;
pub use self::test_client::*;

pub mod traits;

pub mod format_blocks;

pub mod verify;
use self::verify::{verify_and_process_request, team_pointer_to_public_key};

use protocol::*;

pub trait Client: traits::DBConnect + traits::Broadcast + traits::Identify {
    fn read_next_block(&self) -> Result<ReadBlocksResponse> {
        self.read_next_block_from_hash(self.get_last_block_hash()?)
    }

    fn read_next_block_from_hash(&self, last_block_hash: Option<Vec<u8>>) -> Result<ReadBlocksResponse> {
        let response = self.broadcast::<ReadBlocksResponse>(
            &Endpoint::Sigchain,
            &self.read_block_request(last_block_hash.clone())?
        )?;

        self.verify_and_save_read_response(&response, last_block_hash)?;

        Ok(response)
    }

    fn read_new_block_from_hash_with_key(
        &self,
        last_block_hash: Option<Vec<u8>>,
        read_key_pair: &SignKeyPair,
    ) -> Result<ReadBlocksResponse> {
        let response = self.broadcast::<ReadBlocksResponse>(
            &Endpoint::Sigchain,
            &self.read_block_request_with_key(last_block_hash.clone(), &read_key_pair)?
        )?;

        self.verify_and_save_read_response(&response, last_block_hash)?;

        Ok(response)
    }

    fn verify_and_save_read_response(
        &self,
        response: &ReadBlocksResponse,
        mut last_block_hash: Option<Vec<u8>>
    ) -> Result<()> {
        for block in &response.blocks {
            let payload = self.verified_payload_with_db_txn(block)?;
            let incoming_last_block_hash = match payload.body {
                Body::Main(main_chain) => {
                    main_chain.last_block_hash()
                }
                _ => {
                    bail!("not main chain body")
                }
            };
            if incoming_last_block_hash != last_block_hash {
                return Err("last_block_hash does not match".into());
            }
            last_block_hash = Some(block.payload_hash());
        }
        Ok(())
    }

    fn read_block_request(&self, last_block_hash: Option<Vec<u8>>) -> Result<SignedMessage>;

    fn read_block_request_with_key(&self, last_block_hash: Option<Vec<u8>>, sign_key_pair: &SignKeyPair) -> Result<SignedMessage>;

    fn verified_payload_with_db_txn(&self, request: &protocol::SignedMessage) -> Result<Message> {
        self.db_conn().transaction::<_, Error, _>(|| {
            self.verified_payload(request)
        })
    }

    fn process_wrapped_key(
        &self,
        conn: &db::TeamDBConnection,
        logger_identity_public_key: &[u8],
        wrapped_key: &logging::WrappedKey,
    ) -> Result<()> {
        if wrapped_key.recipient_public_key == self.get_encryption_public_key(&self.identity_pk())? {
            match self.unwrap_log_encryption_key(
                &BoxedMessage{
                    recipient_public_key: wrapped_key.recipient_public_key.clone(),
                    ciphertext: wrapped_key.ciphertext.clone(),
                    sender_public_key: self.get_encryption_public_key(logger_identity_public_key)?,
                }
            ) {
                Ok(symmetric_encryption_key) => {
                    db::LogChain::update_symmetric_encryption_key(
                        conn,
                        logger_identity_public_key,
                        Some(symmetric_encryption_key),
                    )?;
                }
                Err(e) => {
                    // Ignore encryption/encoding errors caused by member sending malformed wrapped key.
                    error!("failed to unwrap log encryption key {:?}", e);
                }
            }
        }

        Ok(())
    }

    fn process_encrypted_log(
        &self,
        conn: &db::TeamDBConnection,
        logger_identity_public_key: &[u8],
        encrypted_log: EncryptedLog
    ) -> Result<()> {

        if let Some(symmetric_key) = db::LogChain::find(conn, logger_identity_public_key)?.symmetric_encryption_key {
            let result = || -> Result<(Log, String)> {
                let plaintext_log = secretbox::decrypt(symmetric_key, encrypted_log.ciphertext)?;
                let log_json = std::str::from_utf8(&plaintext_log)?;
                let log: Log = serde_json::from_str(log_json)?;
                Ok((log, log_json.into()))
            }();

            if let Ok((log, log_json)) = result {
                db::Log {
                    team_public_key: self.team_pk().into(),
                    member_public_key: logger_identity_public_key.to_vec(),
                    log_json: log_json.to_string(),
                    unix_seconds: log.unix_seconds as i64,
                }.insert(conn.conn)?;
            } else {
                // Ignore encryption/encoding errors caused by member sending malformed log.
                error!("failed to process encrypted log {:?}", result);
            }
        }

        Ok(())
    }

    fn verified_payload(&self, block: &SignedMessage) -> Result<Message> {
        let unverified_message : Message = serde_json::from_str(&block.message)?;
        let conn = &self.team_db_conn();

        // Client PRE-processing / verification.
        match &unverified_message.body {
            &Body::Main(ref main_chain) => {
                use MainChain::*;
                match main_chain {
                    &Create(ref create_chain) => {
                        if *create_chain.creator_identity.public_key != *self.team_pk() {
                            bail!("team_public_key does not match");
                        }
                    },
                    &Read(_) => {
                        bail!("unexpected ReadBlocks")
                    }
                    &Append(ref _append_block) => {
                        // Note: instead of verifying that an append block is for the current team,
                        // rely on the fact that it must append to a create_block
                        // previously stored by the client.
                    }
                }
            },
            &Body::Log(ref log_chain) => {
                use LogChain::*;
                match log_chain {
                    &Create(ref create_log_chain) => {
                        let team_public_key = team_pointer_to_public_key(conn.conn, &create_log_chain.team_pointer)?;
                        if team_public_key != self.team_pk() {
                            bail!("log chain not part of this team");
                        }
                    }
                    &Append(_)  => {

                    }
                    &Read(_) => {
                        bail!("unexpected ReadLogBlocks")
                    }
                }
            },
            &Body::ReadToken(_) => {
                bail!("unexpected ReadToken")
            },
            &Body::EmailChallenge(_) => {
                bail!("unexpected EmailChallenge")
            },
            &Body::PushSubscription(_) => {
                bail!("unexpected PushSubscription")
            },
            &Body::ReadBillingInfo(_) => {
                bail!("unexpected ReadBillingInfo")
            },
        };

        verify_and_process_request(conn.conn, &block)?;

        // Client POST-processing.
        match &unverified_message.body {
            &Body::Main(ref main_chain) => {
                use MainChain::*;
                use Operation::*;
                use LoggingEndpoint::*;
                match main_chain {
                    &Append(Block{operation: RemoveLoggingEndpoint(CommandEncrypted(_)),..}) => {
                        db::QueuedLog::clear(conn.conn)?;
                    }
                    _ => {}
                }
            },
            &Body::Log(ref log_chain) => {
                use LogChain::*;
                match log_chain {
                    &Create(ref create_log_chain) => {
                        for wrapped_key in &create_log_chain.wrapped_keys {
                            self.process_wrapped_key(conn, &block.public_key, wrapped_key)?;
                        }
                    }
                    &Append(ref append_log_block) => {
                        use LogOperation::*;
                        match &append_log_block.operation {
                            &AddWrappedKeys(ref new_wrapped_keys) => {
                                for wrapped_key in new_wrapped_keys {
                                    self.process_wrapped_key(conn, &block.public_key, wrapped_key)?;
                                }
                                if *block.public_key == *self.identity_pk() {
                                    db::CurrentWrappedKey::add(conn.conn, &new_wrapped_keys.iter()
                                        .map(|wrapped_key| db::CurrentWrappedKey{destination_public_key: wrapped_key.recipient_public_key.clone()})
                                        .collect::<Vec<_>>())?;
                                }
                            },
                            &RotateKey(ref rotated_keys) => {
                                db::LogChain::update_symmetric_encryption_key(
                                    conn,
                                    &block.public_key,
                                    None,
                                )?;

                                for wrapped_key in rotated_keys {
                                    self.process_wrapped_key(conn, &block.public_key, wrapped_key)?;
                                }

                                if *block.public_key == *self.identity_pk() {
                                    db::CurrentWrappedKey::set(conn.conn, &rotated_keys.iter()
                                        .map(|wrapped_key| db::CurrentWrappedKey{destination_public_key: wrapped_key.recipient_public_key.clone()})
                                        .collect::<Vec<_>>())?;
                                }
                            },
                            &EncryptLog(ref encrypted_log) => {
                                self.process_encrypted_log(conn, &block.public_key, encrypted_log.clone())?;
                            },
                        }
                    }
                    &Read(_) => {}
                }
            },
            &Body::ReadToken(_) => {
                bail!("unexpected ReadToken")
            },
            &Body::EmailChallenge(_) => {
                bail!("unexpected EmailChallenge")
            },
            &Body::PushSubscription(_) => {
                bail!("unexpected PushSubscription")
            },
            &Body::ReadBillingInfo(_) => {
                bail!("unexpected ReadBillingInfo")
            },
        };
        Ok(unverified_message)
    }

    fn process_team_filtered_log_blocks_db_txn(&self, response: &ReadTeamLogBlocksResponse) -> Result<()> {
        use sigchain_core::errors::{self, Error, ErrorKind};
        // Clients can end up reading their own blocks again depending on the server's logical timestamp ordering,
        // ignore this failure.
        for block in &response.blocks {
            match self.db_conn().transaction::<_, Error, _>(|| {
                self.verified_payload(block)
            }).map(|_| ()) {
                Err(Error(ErrorKind::Specified(errors::BlockExists), _)) => Ok(()),
                r @ _ => r,
            }?;
        }
        if let Some(update_logical_timestamp) = response.update_logical_timestamp {
            self.db_conn().transaction::<_, Error, _>(|| {
                let mut team = db::Team::find(&self.team_db_conn())?;
                team.last_read_log_chain_logical_timestamp = Some(update_logical_timestamp);
                team.update(self.db_conn())?;
                Ok(())
            })?;
        }
        Ok(())
    }

    fn set_policy(&self, policy: Policy) -> Result<()>;
    fn set_team_info(&self, team_info: TeamInfo) -> Result<()>;
    fn create_invite(&self, restriction: IndirectInvitationRestriction) -> Result<String>;
    fn cancel_invite(&self) -> Result<()>;
    fn remove_member(&self, email: &str) -> Result<()>;
    fn remove_member_pk(&self, public_key: &[u8]) -> Result<()>;
    fn leave(&self) -> Result<()>;
    fn update_team_blocks(&self) -> Result<()>;
    fn update_team_log_blocks(&self) -> Result<()>;
    fn update_my_log_blocks(&self) -> Result<()>;
    fn pin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()>;
    fn unpin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()>;

    fn add_admin(&self, identity_email: &str) -> Result<()>;
    fn remove_admin(&self, identity_email: &str) -> Result<()>;
    fn add_admin_pk(&self, public_key: &[u8]) -> Result<()>;
    fn remove_admin_pk(&self, public_key: &[u8]) -> Result<()>;

    fn enable_logging(&self) -> Result<()>;
    fn disable_logging(&self) -> Result<()>;
    fn unwrap_log_encryption_key(&self, wrapped_key: &logging::BoxedMessage) -> Result<Vec<u8>>;

    fn request_billing_info(&self) -> Result<billing::BillingInfo>;
}

pub trait OwnedKeyPair: traits::DBConnect + traits::Broadcast + traits::Identify {
    fn commit_send<R: serde::de::DeserializeOwned>(&self, endpoint: &Endpoint, signed_message: &SignedMessage) -> Result<R>;
    fn sign_commit_send<R: serde::de::DeserializeOwned>(&self, endpoint: &Endpoint, payload: &Body) -> Result<R> {
        let signed_message = &SignedMessage::from_message(Message::new(payload.clone()), self.sign_key_pair())?;
        self.commit_send(endpoint, &signed_message)
    }
    fn sign_send<R: serde::de::DeserializeOwned>(&self, endpoint: &Endpoint, payload: &Body) -> Result<R> {
        let signed_message = &SignedMessage::from_message(Message::new(payload.clone()), self.sign_key_pair())?;
        self.broadcast(endpoint, &signed_message)
    }
    fn sign_key_pair(&self) -> &SignKeyPair;
    fn box_public_key(&self) -> &box_::ed25519_box::PublicKey;
    fn box_secret_key(&self) -> &box_::ed25519_box::SecretKey;
    fn create_request(&self, op: Operation) -> Result<Body> {
        let last_block_hash = match self.get_last_block_hash()? {
            Some(last_block_hash) => last_block_hash,
            None => bail!("start of hash chain unknown"),
        };
        self.create_request_with_hash(op, last_block_hash)
    }

    fn create_request_with_hash(&self, op: Operation, last_block_hash: Vec<u8>) -> Result<Body> {
        let write_block = Block {
            last_block_hash: last_block_hash.into(),
            operation: op,
        };
        let body = Body::Main(MainChain::Append(write_block.clone()));
        Ok(body)
    }

    fn create_team_request(&self, name: &str, creator: team::Identity) -> Result<Body> {
        let create_chain = GenesisBlock {
            team_info: TeamInfo {
                name: name.clone().into(),
            },
            creator_identity: creator,
        };
        let body = Body::Main(MainChain::Create(create_chain.clone()));
        Ok(body)
    }

    fn create_direct_invite(&self, direct_invite: team::DirectInvitation) -> Result<()> {
        let invite_request = self.create_request(Operation::Invite(Invitation::Direct(direct_invite)))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &invite_request)?;
        Ok(())
    }
    fn accept_direct_invite(&self, identity: Identity) -> Result<()> {
        let request = self.create_request(
            Operation::AcceptInvite(identity))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }

    //  LOGGING
    fn create_log_chain_if_not_exists(&self) -> Result<()> {
        time_fn!("create_log_chain_if_not_exists");
        use protocol::Body::Log;
        use logging::{GenesisLogBlock};
        use logging::LogChain::*;

        let conn = self.db_conn();
        let team_conn = &db::TeamDBConnection{conn, team: self.team_pk()};

        match db::LogChain::find(team_conn, self.identity_pk()).optional()? {
            Some(_) => {},
            None => {
                let create_log_chain = Log(Create(GenesisLogBlock {
                    team_pointer: self.team_pointer()?,
                    wrapped_keys: vec![],
                }));
                self.sign_commit_send::<E>(&Endpoint::Sigchain, &create_log_chain)?;
            }
        };
        Ok(())
    }
    fn wrap_keys_if_admins_changed(&self) -> Result<()> {
        time_fn!("wrap_keys_if_admin_changed");
        use protocol::Body::Log;
        use logging::{LogBlock};
        use logging::LogOperation::*;
        use logging::LogChain::*;

        use std::collections::HashSet;
        use std::iter::FromIterator;
        use crypto::ed25519_box::PublicKey;

        let conn = self.db_conn();
        let team_conn = &db::TeamDBConnection{conn, team: self.team_pk()};

        let current_log_destination_pks = HashSet::<Vec<u8>>::from_iter(
            db::CurrentWrappedKey::all(conn)?.into_iter().map(|k| k.destination_public_key)
        );

        let expected_log_destination_pks = HashSet::<Vec<u8>>::from_iter(
            self.get_admins()?.into_iter()
                .map(|i| i.encryption_public_key)
                .chain(Some(self.box_public_key().0.to_vec()))
        );

        use logging::{PlaintextBody};

        let new_wrapped_keys = {
            if current_log_destination_pks == expected_log_destination_pks {
                None
            } else if (&current_log_destination_pks - &expected_log_destination_pks).is_empty() {
                //  noone removed
                let symmetric_key = db::LogChain::find(team_conn, self.identity_pk())?.symmetric_encryption_key.unwrap_or(secretbox::gen());
                Some(AddWrappedKeys(
                    (&expected_log_destination_pks - &current_log_destination_pks).iter().filter_map(|box_pk|{
                        PublicKey::from_slice(box_pk)
                    }).map(|box_pk| {
                        let body = serde_json::to_vec(&PlaintextBody::LogEncryptionKey(symmetric_key.clone()))
                            .map_err(|e| { error!("{:?}", e); e })?;
                        Ok(WrappedKey {
                            ciphertext: box_::seal(&body, self.box_secret_key(), &box_pk)?,
                            recipient_public_key: box_pk.0.to_vec(),
                        })
                    }).filter_map(Result::ok).collect::<Vec<_>>()
                ))
            } else {
                //  someone removed
                let symmetric_key = secretbox::gen();
                Some(RotateKey(
                    expected_log_destination_pks.iter().filter_map(|box_pk|{
                        PublicKey::from_slice(box_pk)
                    }).map(|box_pk| {
                        let body = serde_json::to_vec(&PlaintextBody::LogEncryptionKey(symmetric_key.clone()))
                            .map_err(|e| { error!("{:?}", e); e })?;
                        Ok(WrappedKey {
                            ciphertext: box_::seal(&body, self.box_secret_key(), &box_pk)?,
                            recipient_public_key: box_pk.0.to_vec(),
                        })
                    }).filter_map(Result::ok).collect::<Vec<_>>()
                ))
            }
        };

        if let Some(new_wrapped_keys) = new_wrapped_keys {
            let wrapped_key_block = Log(Append(LogBlock{
                last_block_hash: db::LogChain::find(team_conn, self.identity_pk())?.last_block_hash,
                operation: new_wrapped_keys,
            }));
            self.sign_commit_send::<E>(&Endpoint::Sigchain, &wrapped_key_block)?;
        }
        Ok(())
    }
    fn encrypt_log(&self, log: logs::Log) -> Result<()> {
        use protocol::Body::Log;
        use logging::{LogBlock};
        use logging::LogOperation::*;
        use logging::LogChain::*;

        let conn = self.db_conn();
        let team_conn = &db::TeamDBConnection{conn, team: self.team_pk()};

        if !self.is_command_encrypted_logging_enabled()? {
            return Ok(())
        }

        conn.transaction::<_, Error, _>(|| {
            db::QueuedLog::add(conn, &db::NewQueuedLog{log_json: serde_json::to_vec(&log)?})?;
            Ok(())
        })?;

        conn.transaction(|| {
            self.create_log_chain_if_not_exists()
        })?;

        conn.transaction(|| {
            self.wrap_keys_if_admins_changed()
        })?;

        while db::QueuedLog::any(conn)? {
            conn.transaction::<_, Error, _>(|| {
                let log = db::QueuedLog::next(conn)?;
                let log_chain = db::LogChain::find(team_conn, self.identity_pk())?;
                let encrypted_log = Log(Append(LogBlock{
                    last_block_hash: log_chain.last_block_hash,
                    operation: EncryptLog(EncryptedLog{
                        ciphertext: secretbox::encrypt(
                            &log.log_json,
                            &log_chain.symmetric_encryption_key.ok_or("no symmetric key")?,
                        )?,
                    }),
                }));
                self.sign_commit_send::<E>(&Endpoint::Sigchain, &encrypted_log)?;
                log.remove(conn)?;
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl <T: traits::DBConnect + traits::Broadcast + OwnedKeyPair> Client for T {
    fn read_block_request(&self, last_block_hash: Option<Vec<u8>>) -> Result<SignedMessage> {
        use TeamPointer::*;
        let body = Body::Main(MainChain::Read(
            ReadBlocksRequest {
                team_pointer: match last_block_hash {
                    Some(last_block_hash) => LastBlockHash(last_block_hash),
                    None => PublicKey(self.team_pk().into()),
                },
                nonce: random_nonce()?,
                token: None,
            }));
        SignedMessage::from_message(Message::new(body), self.sign_key_pair())
    }
    fn read_block_request_with_key(&self, last_block_hash: Option<Vec<u8>>, sign_key_pair: &SignKeyPair) -> Result<SignedMessage> {
        use TeamPointer::*;
        let body = Body::Main(MainChain::Read(
            ReadBlocksRequest {
                team_pointer: match last_block_hash {
                    Some(last_block_hash) => LastBlockHash(last_block_hash),
                    None => PublicKey(self.team_pk().into()),
                },
                nonce: random_nonce()?,
                token: None,
            }));
        SignedMessage::from_message(Message::new(body), &sign_key_pair)
    }
    fn set_policy(&self, policy: Policy) -> Result<()> {
        let set_policy_request = self.create_request(
            Operation::SetPolicy(policy.clone()))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &set_policy_request)?;
        Ok(())
    }
    fn set_team_info(&self, team_info: TeamInfo) -> Result<()> {
        let set_team_info_request = self.create_request(
            Operation::SetTeamInfo(team_info))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &set_team_info_request)?;
        Ok(())
    }
    fn create_invite(&self, restriction: IndirectInvitationRestriction) -> Result<String> {
        let nonce_keypair_seed = gen_sign_key_pair_seed()?;
        let nonce_keypair = sign_keypair_from_seed(&nonce_keypair_seed)?;

        let last_block_hash: Vec<u8> = self.get_last_block_hash()?
            .ok_or("no last_block_hash")?;

        let (invitation, secret_invite_link) = team::IndirectInvitation::create_link(
            nonce_keypair.public_key_bytes().into(),
            team::IndirectInvitationSecret {
                initial_team_public_key: self.team_pk().into(),
                last_block_hash,
                nonce_keypair_seed,
                restriction,
            },
        )?;

        let invite_request = self.create_request(Operation::Invite(Invitation::Indirect(invitation)))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &invite_request)?;

        Ok(secret_invite_link)
    }
    fn cancel_invite(&self) -> Result<()> {
        let cancel_request = self.create_request(Operation::CloseInvitations(E {}))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &cancel_request)?;
        Ok(())
    }
    fn remove_member(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        self.remove_member_pk(&matching_public_key)
    }
    fn remove_member_pk(&self, public_key: &[u8]) -> Result<()> {
        let request = self.create_request(
            Operation::Remove(public_key.into()))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn leave(&self) -> Result<()> {
        let request = self.create_request(Operation::Leave(E{}))?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn update_team_blocks(&self) -> Result<()> {
        while self.read_next_block()?.more {}
        Ok(())
    }
    fn update_team_log_blocks(&self) -> Result<()> {
        loop {
            let team = db::Team::find(&self.team_db_conn())?;
            let last_logical_ts = team.last_read_log_chain_logical_timestamp.unwrap_or(0);
            let payload = Body::Log(LogChain::Read(ReadLogBlocksRequest {
                filter: LogFilter::Team(
                    TeamLogFilter{
                        team_public_key: self.team_pk().into(),
                        last_logical_timestamp: last_logical_ts,
                    }
                ),
                nonce: random_nonce()?,
                token: None,
            }));
            let resp = self.sign_send::<ReadTeamLogBlocksResponse>(&Endpoint::Sigchain, &payload)?;

            self.process_team_filtered_log_blocks_db_txn(&resp)?;

            if !resp.more {
                break
            }
        }

        Ok(())
    }
    fn update_my_log_blocks(&self) -> Result<()> {
        loop {
            let payload = Body::Log(LogChain::Read(ReadLogBlocksRequest {
                filter: LogFilter::Member(self.my_log_pointer()?),
                nonce: random_nonce()?,
                token: None,
            }));
            let resp = self.sign_send::<ReadMemberLogBlocksResponse>(&Endpoint::Sigchain, &payload)?;

            for block in &resp.blocks {
                self.verified_payload_with_db_txn(block)?;
            }

            if !resp.more {
                break
            }
        }

        Ok(())
    }
    fn pin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()> {
        let op = Operation::PinHostKey(SSHHostKey{
            host: host.into(),
            public_key: public_key.into(),
        });
        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn unpin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()> {
        let op = Operation::UnpinHostKey(SSHHostKey{
            host: host.into(),
            public_key: public_key.into(),
        });

        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn add_admin(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        let op = Operation::Promote(matching_public_key);
        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn remove_admin(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        let op = Operation::Demote(matching_public_key);
        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn add_admin_pk(&self, public_key: &[u8]) -> Result<()> {
        let op = Operation::Promote(public_key.into());
        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn remove_admin_pk(&self, public_key: &[u8]) -> Result<()> {
        let op = Operation::Demote(public_key.into());
        let request = self.create_request(op)?;
        self.sign_commit_send::<E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn enable_logging(&self) -> Result<()> {
        use LoggingEndpoint::*;
        let op = Operation::AddLoggingEndpoint(CommandEncrypted(E{}));

        let request = self.create_request(op)?;
        self.sign_commit_send::<team::E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn disable_logging(&self) -> Result<()> {
        use LoggingEndpoint::*;
        let op = Operation::RemoveLoggingEndpoint(CommandEncrypted(E{}));

        let request = self.create_request(op)?;
        self.sign_commit_send::<team::E>(&Endpoint::Sigchain, &request)?;
        Ok(())
    }
    fn unwrap_log_encryption_key(&self, wrapped_key: &logging::BoxedMessage) -> Result<Vec<u8>> {
        let conn = &db::TeamDBConnection{team: self.team_pk(), conn: self.db_conn()};

        // Verify sender is or used to be on team
        let identities = db::Identity::filter_by_encryption_public_key(
            conn, &wrapped_key.sender_public_key)?;

        if identities.len() == 0 {
            bail!("No identity with given encryption public key")
        }

        use crypto;
        let plaintext = crypto::box_::open(&wrapped_key.ciphertext, &wrapped_key.sender_public_key, self.box_secret_key())?;
        let plaintext_body : logging::PlaintextBody = serde_json::from_slice(&plaintext)?;
        match plaintext_body {
            logging::PlaintextBody::LogEncryptionKey(symmetric_key) =>  Ok(symmetric_key),
        }
    }
    fn request_billing_info(&self) -> Result<billing::BillingInfo> {
        use sigchain_core;
        if !self.is_admin()? {
            bail!(sigchain_core::errors::NotAnAdmin)
        }
        let body = Body::ReadBillingInfo(billing::ReadBillingInfo {
            team_public_key: self.team_pk().into(),
            token: None,
        });
        let request = SignedMessage::from_message(Message::new(body), self.sign_key_pair())?;
        self.broadcast::<billing::BillingInfo>(&Endpoint::BillingInfo, &request)
    }
}
