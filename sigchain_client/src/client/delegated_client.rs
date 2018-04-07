extern crate reqwest;
use super::*;
use enclave_protocol::*;
use team::SignedReadToken;
use errors::{Result, Error};
use verify::map_read_token_to_identity_pk;

/// Admin client that routes requests to the currently paired phone (through `krd`) to perform
/// private key signatures and decryptions.
#[allow(dead_code)]
pub struct DelegatedNetworkClient {
    pub conn: DBConnection,
    pub public_key_and_checkpoint: TeamCheckpoint,
    pub should_read_notify_logs: bool,
}

impl DelegatedNetworkClient {
    pub fn from_public_key_and_checkpoint(public_key_and_checkpoint: TeamCheckpoint) -> Result<DelegatedNetworkClient> {
        Ok(DelegatedNetworkClient {
            conn: NetworkClient::make_db_conn()?,
            public_key_and_checkpoint,
            should_read_notify_logs: false,
        })
    }
}

use traits::{Broadcast, DBConnect, Identify};
impl Broadcast for DelegatedNetworkClient {
    fn broadcast<T>(&self, endpoint: &protocol::Endpoint, request: &SignedMessage) -> Result<T> where
        T: serde::de::DeserializeOwned {
        network_client::default_broadcast(&self.public_key_and_checkpoint.server_endpoints, endpoint, request)
    }
}

impl DBConnect for DelegatedNetworkClient {
    fn db_conn(&self) -> &DBConnection {
        &self.conn
    }
}

impl Identify for DelegatedNetworkClient {
    fn identity_pk(&self) -> &[u8] {
        &self.public_key_and_checkpoint.public_key
    }
    fn team_pk(&self) -> &[u8] {
        &self.public_key_and_checkpoint.team_public_key
    }
}

use enclave_protocol::RequestableTeamOperation;
impl Client for DelegatedNetworkClient {
    fn read_block_request(&self, last_block_hash: Option<Vec<u8>>) -> Result<SignedMessage> {
        let read_token = self.get_or_request_read_token()?;
        let signed_read_token = serde_json::from_slice(&read_token.token)?;
        let reader_key_pair : SignKeyPair = serde_json::from_slice(&read_token.reader_key_pair)?;

        use team::{MainChain};
        use protocol::{SignedMessage};
        use TeamPointer::*;
        let body = Body::Main(MainChain::Read(
            ReadBlocksRequest {
                team_pointer: match last_block_hash {
                    Some(last_block_hash) => LastBlockHash(last_block_hash),
                    None => PublicKey(self.team_pk().into()),
                },
                nonce: random_nonce()?,
                token: Some(signed_read_token),
            }));

        let request = SignedMessage::from_message(Message::new(body.clone()), &reader_key_pair)?;
        Ok(request)
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
        let request = SignedMessage::from_message(Message::new(body), &sign_key_pair)?;
        Ok(request)
    }

    fn set_policy(&self, policy: Policy) -> Result<()> {
        self.request_operation(RequestableTeamOperation::SetPolicy(policy.clone()))?;
        Ok(())
    }

    fn set_team_info(&self, team_info: TeamInfo) -> Result<()> {
        self.request_operation(RequestableTeamOperation::SetTeamInfo(team_info))?;
        Ok(())
    }

    fn create_invite(&self, restriction: IndirectInvitationRestriction) -> Result<String> {
        let response = self.request_operation(RequestableTeamOperation::IndirectInvite(restriction))?;
        let invite = match response.data {
            Some(enclave_protocol::TeamOperationResponseData::InviteLink(invite)) => invite,
            _ => bail!("no invite link returned"),
        };
        Ok(invite)
    }
    fn cancel_invite(&self) -> Result<()> {
        self.request_operation(RequestableTeamOperation::CloseInvitations(E{}))?;
        Ok(())
    }
    fn remove_member(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        let op = RequestableTeamOperation::Remove(matching_public_key);
        self.request_operation(op)?;
        Ok(())
    }
    fn remove_member_pk(&self, public_key: &[u8]) -> Result<()> {
        let op = RequestableTeamOperation::Remove(public_key.into());
        self.request_operation(op)?;
        Ok(())
    }
    fn leave(&self) -> Result<()> {
        let op = RequestableTeamOperation::Leave(E{});
        self.request_operation(op)?;
        Ok(())
    }

    fn update_team_blocks(&self) -> Result<()> {
        info!("updating team blocks");
        while self.read_next_block()?.more {}
        if db::Block::find(self.db_conn(), &self.public_key_and_checkpoint.last_block_hash).optional()?.is_none() {
            use sigchain_core;
            bail!(sigchain_core::errors::TeamCheckpointLastBlockHashNotReached)
        }
        Ok(())
    }

    fn update_team_log_blocks(&self) -> Result<()> {
        self.update_team_log_blocks_with_limit(None).map(|_| ())
    }
    fn update_my_log_blocks(&self) -> Result<()> {
        unimplemented!()
    }
    fn pin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()> {
        let op = RequestableTeamOperation::PinHostKey(SSHHostKey{
            host: host.into(),
            public_key: public_key.into(),
        });
        self.request_operation(op)?;
        Ok(())
    }
    fn unpin_host_key(&self, host: &str, public_key: &[u8]) -> Result<()> {
        let op = RequestableTeamOperation::UnpinHostKey(SSHHostKey{
            host: host.into(),
            public_key: public_key.into(),
        });
        self.request_operation(op)?;
        Ok(())
    }
    fn add_admin(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        let op = RequestableTeamOperation::Promote(matching_public_key);
        self.request_operation(op)?;
        Ok(())
    }
    fn remove_admin(&self, identity_email: &str) -> Result<()> {
        let matching_public_key = self.get_active_member_by_email(identity_email)?.public_key;
        let op = RequestableTeamOperation::Demote(matching_public_key);
        self.request_operation(op)?;
        Ok(())
    }
    fn add_admin_pk(&self, public_key: &[u8]) -> Result<()> {
        let op = RequestableTeamOperation::Promote(public_key.into());
        self.request_operation(op)?;
        Ok(())
    }
    fn remove_admin_pk(&self, public_key: &[u8]) -> Result<()> {
        let op = RequestableTeamOperation::Demote(public_key.into());
        self.request_operation(op)?;
        Ok(())
    }
    fn enable_logging(&self) -> Result<()> {
        use LoggingEndpoint::*;
        let op = RequestableTeamOperation::AddLoggingEndpoint(CommandEncrypted(E{}));
        self.request_operation(op)?;
        Ok(())
    }
    fn disable_logging(&self) -> Result<()> {
        use LoggingEndpoint::*;
        let op = RequestableTeamOperation::RemoveLoggingEndpoint(CommandEncrypted(E{}));
        self.request_operation(op)?;
        Ok(())
    }
    fn unwrap_log_encryption_key(&self, wrapped_key: &logging::BoxedMessage) -> Result<Vec<u8>> {
        use enclave_protocol::{RequestBody, LogDecryptionRequest};
        let response = krd_client::daemon_enclave_control_request(
            &enclave_protocol::Request::new(
                RequestBody::LogDecryptionRequest(LogDecryptionRequest{
                    wrapped_key: wrapped_key.clone(),
                }))?,
            self.should_read_notify_logs
        )?;

        use ResponseBody::LogDecryptionResponse;
        match response.body {
            LogDecryptionResponse(result) => {
                use enclave_protocol::Result::*;
                match result {
                    Success(log_decryption_response) => {
                        return Ok(log_decryption_response.log_decryption_key);
                    }
                    Error(e) => bail!("{:?}", e),
                }
            }
            _ =>  bail!("no LogDecryptionResponse returned"),
        }
    }
    fn request_billing_info(&self) -> Result<billing::BillingInfo> {
        use sigchain_core;
        if !self.is_admin()? {
            bail!(sigchain_core::errors::NotAnAdmin)
        }
        use protocol::{SignedMessage};
        let read_token = self.get_or_request_read_token()?;
        let signed_read_token : SignedReadToken = serde_json::from_slice(&read_token.token)?;
        let reader_key_pair : SignKeyPair = serde_json::from_slice(&read_token.reader_key_pair)?;

        let body = Body::ReadBillingInfo(billing::ReadBillingInfo {
            team_public_key: self.team_pk().into(),
            token: Some(signed_read_token),
        });
        let request = SignedMessage::from_message(Message::new(body), &reader_key_pair)?;
        self.broadcast::<billing::BillingInfo>(&Endpoint::BillingInfo, &request)
    }
}

impl DelegatedNetworkClient {
    pub fn get_current_team_checkpoint(db_conn: &DBConnection) -> Result<TeamCheckpoint> {
        if let Some(team_checkpoint) = krd_client::daemon_me_request()?.team_checkpoint {
            let team_pk = team_checkpoint.team_public_key.clone();
            let identity_pk = team_checkpoint.public_key.clone();

            if DelegatedNetworkClient::get_read_token_for_team_and_identity(db_conn, &team_pk, &identity_pk)?.is_some() {
                return Ok(team_checkpoint);
            }
        }

        // if we do NOT have a read token, force a refresh to
        // get latest information about the user's team identity
        return krd_client::daemon_me_request_force_refresh()?.team_checkpoint.ok_or("no team checkpoint returned".into());
    }

    pub fn for_current_team() -> Result<DelegatedNetworkClient> {
        let conn = NetworkClient::make_db_conn()?;
        let team_checkpoint = DelegatedNetworkClient::get_current_team_checkpoint(&conn)?;

        Ok(DelegatedNetworkClient {
            conn,
            public_key_and_checkpoint: team_checkpoint,
            should_read_notify_logs: false,
        })
    }

    pub fn has_team_or_identity_changed(&self) -> Result<bool> {
        // first get the profile team checkpoint
        // this checks that there's a read token or does a force refresh from the phone
        let team_checkpoint = DelegatedNetworkClient::get_current_team_checkpoint(self.db_conn())?;

        // ensure that the team is the same AND the identity is the same
        if  self.team_pk() == team_checkpoint.team_public_key.as_slice() &&
            self.identity_pk() == team_checkpoint.public_key.as_slice()
            {
                return Ok(false);
            }

        return Ok(true)
    }

    pub fn get_read_token(&self) -> Result<Option<db::ReadToken>> {
        DelegatedNetworkClient::get_read_token_for_team_and_identity(self.db_conn(),
                                                                     self.team_pk(),
                                                                     self.identity_pk())
    }

    pub fn get_read_token_for_team_and_identity(conn: &DBConnection, team_public_key: &[u8], identity_public_key: &[u8]) -> Result<Option<db::ReadToken>> {
        conn.transaction::<_, Error, _>(|| {
            if let Some(read_token) = db::ReadToken::find(conn, team_public_key).optional()? {
                let reader_key_pair : SignKeyPair = serde_json::from_slice(&read_token.reader_key_pair)?;
                let signed_read_token: SignedMessage = serde_json::from_slice(&read_token.token)?;

                match map_read_token_to_identity_pk(&reader_key_pair.public_key.0, &Some(signed_read_token)) {
                    Ok(member_public_key) => {
                        // ensure that this read token is for the current client's identity
                        if member_public_key.as_slice() == identity_public_key {
                            return Ok(Some(read_token))
                        }
                        info!("ReadToken is for a different client identity");
                    },
                    Err(e) => {
                        info!("ReadToken invalid: {}", e);
                    }
                };
            }

            Ok(None)
        })
    }

    fn get_or_request_read_token(&self) -> Result<db::ReadToken> {
        if let Some(read_token) = self.get_read_token()? {
            return Ok(read_token);
        }
        let conn = self.db_conn();
        conn.transaction::<_, Error, _>(|| -> Result<db::ReadToken> {
            db::ReadToken::delete(conn, &self.team_pk())?;

            let reader_key_pair = crypto::sign::gen_sign_key_pair()?;
            use enclave_protocol::{ReadTeamRequest, Request, RequestBody};
            let read_request = Request::new(RequestBody::ReadTeamRequest(ReadTeamRequest{
                public_key: reader_key_pair.public_key_bytes().into(),
            }))?;

            let response = krd_client::daemon_enclave_control_request(&read_request, self.should_read_notify_logs)?;

            use ResponseBody::ReadTeamResponse;
            use enclave_protocol::Result::{Success, Error};
            match response.body {
                ReadTeamResponse(result) => {
                    match result {
                        Success(signed_read_token) => {
                            let complete_read_token = db::ReadToken {
                                team_public_key: self.team_pk().into(),
                                token: serde_json::to_vec(&signed_read_token)?,
                                reader_key_pair: serde_json::to_vec(&reader_key_pair)?,
                            };
                            complete_read_token.insert(conn)?;
                            return Ok(complete_read_token);
                        }
                        Error(e) => bail!("{}", e.error),
                    }
                }
                _ => bail!("{:?}", "no ReadTeamResponse returned"),
            }
        })
    }

    // Returns false if there are more results, but the limit was reached
    pub fn update_team_log_blocks_with_limit(&self, request_limit: Option<u64>) -> Result<bool> {
        info!("updating log blocks (lim {:?})", request_limit);

        use protocol::{SignedMessage};
        use logging::{ReadTeamLogBlocksResponse, LogFilter, ReadLogBlocksRequest};

        let read_token = self.get_or_request_read_token()?;

        let signed_read_token : SignedReadToken = serde_json::from_slice(&read_token.token)?;
        let reader_key_pair : SignKeyPair = serde_json::from_slice(&read_token.reader_key_pair)?;

        let mut request_count = 0;
        let mut all_fetched = true;

        loop {
            let last_logical_ts = db::Team::find(&self.team_db_conn())?
                .last_read_log_chain_logical_timestamp.unwrap_or(0);
            let body = Body::Log(LogChain::Read(ReadLogBlocksRequest {
                filter: LogFilter::Team(
                    TeamLogFilter {
                        team_public_key: self.team_pk().into(),
                        last_logical_timestamp: last_logical_ts,
                    }
                ),
                nonce: random_nonce()?,
                token: Some(signed_read_token.clone()),
            }));
            let request = SignedMessage::from_message(Message::new(body.clone()), &reader_key_pair)?;
            let resp = self.broadcast::<ReadTeamLogBlocksResponse>(&Endpoint::Sigchain, &request)?;

            self.process_team_filtered_log_blocks_db_txn(&resp)?;

            if !resp.more {
                break
            }

            request_count += 1;

            // break out if we've passed the limit number of requests
            if request_limit.map(|limit| request_count >= limit).unwrap_or(false)  {
                all_fetched = false;
                break
            }
        }
        Ok(all_fetched)
    }

    pub fn request_operation(&self, operation: RequestableTeamOperation) -> Result<TeamOperationResponse> {
        use enclave_protocol::{Request, RequestBody, ResponseBody};
        let request = Request::new(RequestBody::TeamOperationRequest(TeamOperationRequest{
            operation,
        }))?;
        let response = krd_client::daemon_enclave_control_request(&request, self.should_read_notify_logs)?;

        use enclave_protocol::Result::*;
        let team_operation_response = match response.body {
            ResponseBody::TeamOperationResponse(result) => {
                match result {
                    Success(team_operation_response) => team_operation_response,
                    //TODO: structure this into an error enum
                    Error(e) => bail!(e.error),
                }
            }
            _ => bail!("{:?}", "no TeamOperationResponse returned"),
        };

        while self.read_next_block()?.more {}

        if !db::Block::exists(self.db_conn(), &team_operation_response.posted_block_hash)? {
            bail!("posted_block_hash not found");
        }

        Ok(team_operation_response)
    }
}
