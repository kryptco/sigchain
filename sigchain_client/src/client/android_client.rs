pub extern crate reqwest;
use super::*;
use enclave_protocol;

use enclave_protocol::{ServerEndpoints, TeamCheckpoint};

use errors::Error;
use client::traits::{DBConnect, Broadcast, Identify};

use std::sync::Arc;

#[allow(dead_code)]
pub struct AndroidClient {
    pub key_pair: SignKeyPair,
    pub box_key_pair: BoxKeyPair,
    pub team_checkpoint: TeamCheckpoint,
    pub http_client: Arc<reqwest::Client>,
    pub db_connection: DBConnection,
}

impl traits::Broadcast for AndroidClient {
    fn broadcast<'a, T>(&self, endpoint: &protocol::Endpoint, request: &SignedMessage) -> super::Result<T>
        where T: serde::de::DeserializeOwned {
        use team::Response;
        time_fn!("request");
        let request_body = serde_json::to_vec(request)?;
        let response_bytes = self.http_client.put(&self.server_endpoints().url(endpoint))?.body(request_body.to_vec()).send()?;
        let response: Response<T> = serde_json::from_reader(response_bytes)?;
        match response {
            Response::Success(t) => Ok(t),
            Response::Error(s) => bail!(s),
        }
    }
}

//TODO: Refactor these methods into OwnedKeyPair
impl AndroidClient {
    pub fn db_dir_to_db_file(db_dir: &str) -> String {
        (db_dir.to_string() + "/team.db").to_string()
    }
    pub fn db_conn(db_dir: String) -> Result<DBConnection> {
        time_fn!("db_conn");
        use dotenv;
        dotenv().ok();

        let database_url = Self::db_dir_to_db_file(&db_dir);
        let conn = DBConnection::establish(&database_url)?;

        db::run_migrations(&conn)?;
        Ok(conn)
    }
    pub fn server_endpoints(&self) -> &ServerEndpoints {
        &self.team_checkpoint.server_endpoints
    }
    pub fn request_email_challenge(http_client: Arc<reqwest::Client>, server_endpoints: &ServerEndpoints, request: &email::PutSendEmailChallengeRequest) -> super::Result<E> {
        let client = reqwest::Client::new()?;
        let response_bytes = http_client.put(&server_endpoints.url(&Endpoint::ChallengeEmail))?
            .body(serde_json::to_vec(request)?).send()?;
        let response: Response<E> = serde_json::from_reader(response_bytes)?;
        match response {
            Response::Success(t) => Ok(t),
            Response::Error(s) => bail!(s),
        }
    }
    pub fn verify_email(&self, email: &str, email_challenge_nonce: &[u8]) -> Result<E> {
        let response_bytes = self.http_client.put(&self.server_endpoints().url(&Endpoint::VerifyEmail))?
            .body(serde_json::to_vec(
                &SignedMessage::from_message(
                    Message::new(Body::EmailChallenge(
                        EmailChallenge{
                            nonce: email_challenge_nonce.to_owned(),
                        }
                    )),
                    &self.key_pair,
                )?
            )?).send()?;
        let response: Response<E> = serde_json::from_reader(response_bytes)?;
        match response {
            Response::Success(t) => Ok(t),
            Response::Error(s) => bail!(s),
        }
    }
    pub fn execute_requestable_operation(&self, req: &enclave_protocol::RequestableTeamOperation) -> Result<enclave_protocol::TeamOperationResponse> {
        use RequestableTeamOperation::*;
        let data = match req {
            &DirectInvite(ref direct_invite) => {
                self.create_direct_invite(direct_invite.clone())?;
                None
            }
            &IndirectInvite(ref restriction) =>  {
                let invite = self.create_invite(restriction.clone())?;
                Some(enclave_protocol::TeamOperationResponseData::InviteLink(invite))
            }
            &CloseInvitations(_) => {
                self.cancel_invite()?;
                None
            }

            &Remove(ref public_key) => {
                self.remove_member_pk(public_key)?;
                None
            }
            &Leave(_) => {
                let _ = self.leave();
                db::CurrentTeam::delete(self.db_conn())?;
                None
            }

            &SetPolicy(ref policy) => {
                self.set_policy(policy.clone())?;
                None
            }
            &SetTeamInfo(ref team_info) => {
                self.set_team_info(team_info.clone())?;
                None
            }

            &PinHostKey(ref host_key) => {
                self.pin_host_key(&host_key.host, &host_key.public_key)?;
                None
            }
            &UnpinHostKey(ref host_key) => {
                self.unpin_host_key(&host_key.host, &host_key.public_key)?;
                None
            }

            &AddLoggingEndpoint(_) => {
                self.enable_logging()?;
                None
            }
            &RemoveLoggingEndpoint(_) => {
                self.disable_logging()?;
                None
            }

            &Promote(ref public_key) => {
                self.add_admin_pk(public_key)?;
                None
            }
            &Demote(ref public_key) => {
                self.remove_admin_pk(public_key)?;
                None
            }
        };
        Ok(enclave_protocol::TeamOperationResponse{
            posted_block_hash: self.get_last_block_hash()?.ok_or("no last_block_hash")?,
            data,
        })
    }

    pub fn fetch_and_decrypt_invite_ciphertext(http_client: Arc<reqwest::Client>, server_endpoints: &ServerEndpoints, invite_link: &str) -> Result<IndirectInvitationSecret> {
        use base64;
        use url;
        use sha256;
        use crypto;

        let symmetric_key_b64 : String = url::Url::parse(invite_link)?
            .path_segments().ok_or("no url path")?
            .next().ok_or("no first path segment")?.into();

        let symmetric_key = base64::decode_config(&symmetric_key_b64, base64::URL_SAFE)?;

        let symmetric_key_hash = sha256::hash(&symmetric_key).0.to_vec();
        let request = InviteSymmetricKeyHash{symmetric_key_hash};
        let request_body = serde_json::to_vec(&request)?;

        let symmetric_key_hash_response =
            http_client.put(&server_endpoints.url(&Endpoint::InviteLinkCiphertext))?.body(request_body.to_vec()).send()?;
        let symmetric_key_hash_response : Response<InviteSymmetricKeyHashResponse> =
            serde_json::from_reader(symmetric_key_hash_response)?;
        let ciphertext = match symmetric_key_hash_response {
            Response::Success(response) => response.ciphertext,
            Response::Error(e) => bail!(e),
        };

        let invite_secret_plaintext =
            crypto::secretbox::decrypt(symmetric_key.to_vec(), ciphertext)?;
        let invite_secret : IndirectInvitationSecret = serde_json::from_slice(&invite_secret_plaintext)?;

        Ok(invite_secret)
    }

    pub fn bootstrap_using_invite(http_client: Arc<reqwest::Client>, server_endpoints: ServerEndpoints, db_dir: String, invite_secret: IndirectInvitationSecret) -> Result<AndroidClient> {
        let sign_key_pair = gen_sign_key_pair()?;
        let cli = AndroidClient {
            team_checkpoint: TeamCheckpoint{
                team_public_key: invite_secret.initial_team_public_key.clone(),
                last_block_hash: invite_secret.last_block_hash.clone(),
                server_endpoints,
                public_key: sign_key_pair.public_key_bytes().into(),
            },
            key_pair: sign_key_pair,
            box_key_pair: gen_box_key_pair()?,
            db_connection: AndroidClient::db_conn(db_dir)?,
            http_client,
        };

        cli.update_team_blocks_using_invite(&invite_secret)?;

        Ok(cli)
    }

    fn update_team_blocks_using_invite(&self, invite_secret: &IndirectInvitationSecret) -> Result<()> {
        use crypto;

        let nonce_keypair = sign::sign_keypair_from_seed(&invite_secret.nonce_keypair_seed)?;

        loop {
            let read_request = ReadBlocksRequest {
                team_pointer: self.team_pointer()?,
                nonce: crypto::random_nonce()?,
                token: None,
            };

            let signed_request = SignedMessage::from_message(
                Message::new(Body::Main(MainChain::Read(read_request))),
                &nonce_keypair,
            )?;

            let read_response = self.broadcast::<ReadBlocksResponse>(&Endpoint::Sigchain, &signed_request)?;

            for block in read_response.blocks {
                self.verified_payload_with_db_txn(&block)?;
            }
            if !read_response.more {
                break
            }
        }

        use sigchain_core;
        if db::Block::find(self.db_conn(), &invite_secret.last_block_hash).optional()?.is_none() {
            bail!(sigchain_core::errors::InviteLastBlockHashNotReached)
        }

        Ok(())
    }

    pub fn accept_invite(&self, identity: Identity, email_challenge_nonce: Vec<u8>, invite_secret: IndirectInvitationSecret) -> Result<()> {
        use crypto;

        self.verify_email(&identity.email, &email_challenge_nonce)?;

        let nonce_keypair = sign::sign_keypair_from_seed(&invite_secret.nonce_keypair_seed)?;

        self.update_team_blocks_using_invite(&invite_secret)?;

        let accept_invite_request = SignedMessage::from_message(
            Message::new(Body::Main(MainChain::Append(
                Block {
                    last_block_hash: self.get_last_block_hash()?.ok_or("no last_block_hash")?,
                    operation: Operation::AcceptInvite(identity),
                }
            ))),
            &nonce_keypair,
        )?;

        self.db_conn().transaction::<_, Error, _>(|| {
            self.verified_payload(&accept_invite_request)?;
            let accept_invite_result = self.broadcast::<E>(&Endpoint::Sigchain, &accept_invite_request)?;
            Ok(())
        })
    }

    pub fn accept_direct_invite(&self, identity: &Identity, email_challenge_nonce: &[u8]) -> Result<()> {
        use sigchain_core;
        if db::Block::find(self.db_conn(), &self.team_checkpoint.last_block_hash).optional()?.is_none() {
            bail!(sigchain_core::errors::InviteLastBlockHashNotReached)
        }

        self.verify_email(&identity.email, &email_challenge_nonce)?;

        let accept_invite_request = SignedMessage::from_message(
            Message::new(Body::Main(MainChain::Append(
                Block {
                    last_block_hash: self.get_last_block_hash()?.ok_or("no last_block_hash")?,
                    operation: Operation::AcceptInvite(identity.clone()),
                }
            ))),
            &self.key_pair,
        )?;

        self.db_conn().transaction::<_, Error, _>(|| {
            self.verified_payload(&accept_invite_request)?;
            let accept_invite_result = self.broadcast::<E>(&Endpoint::Sigchain, &accept_invite_request)?;
            Ok(())
        })
    }

    pub fn subscribe_to_push_notifications(&self, device: PushDevice) -> Result<()> {
        let push_request = SignedMessage::from_message(
            Message::new(Body::PushSubscription(
                push::PushSubscription{
                    team_pointer: self.team_pointer()?,
                    action: push::PushSubscriptionAction::Subscribe(device),
                }
            )),
            &self.key_pair,
        )?;

        let response_bytes = self.http_client.put(&self.server_endpoints().url(&Endpoint::PushSubscription))?
            .body(serde_json::to_vec(&push_request)?).send()?;
        let response: Response<E> = serde_json::from_reader(response_bytes)?;
        match response {
            Response::Success(t) => Ok(()),
            Response::Error(s) => bail!(s),
        }
    }

    pub fn sign_read_token(&self, reader_public_key: &[u8]) -> Result<team::SignedReadToken> {
        use chrono;
        use std::ops::Add;
        SignedMessage::from_message(
            Message::new(Body::ReadToken(
                team::ReadToken::Time(
                    team::TimeToken{
                        reader_public_key: reader_public_key.into(),
                        expiration: chrono::Utc::now().add(chrono::Duration::hours(6)).timestamp(),
                    }
                )
            )),
            &self.key_pair,
        )
    }

    pub fn get_billing_url(&self) -> Result<String> {
        self.db_conn().transaction(|| {
            self.server_endpoints().billing_url(
                &db::Team::find(&self.team_db_conn())?.name,
                self.team_pk(),
                self.identity_pk(),
                &self.get_my_identity()?.email,
            )
        })
    }
}

impl traits::DBConnect for AndroidClient {
    fn db_conn(&self) -> &DBConnection {
        &self.db_connection
    }
}

impl traits::Identify for AndroidClient {
    fn identity_pk(&self) -> &[u8] {
        self.key_pair.public_key_bytes()
    }
    fn team_pk(&self) -> &[u8] {
        &self.team_checkpoint.team_public_key
    }
}

impl OwnedKeyPair for AndroidClient {
    fn commit_send<R: serde::de::DeserializeOwned>(&self, endpoint: &protocol::Endpoint, signed_message: &SignedMessage) -> Result<R> {
        use traits::{Broadcast, DBConnect};
        self.db_conn().transaction::<_, Error, _>(|| {
            self.verified_payload(signed_message)?;
            self.broadcast::<R>(endpoint, signed_message)
        })
    }
    fn sign_key_pair(&self) -> &SignKeyPair {
        &self.key_pair
    }
    fn box_public_key(&self) -> &box_::ed25519_box::PublicKey {
        &self.box_key_pair.public_key
    }
    fn box_secret_key(&self) -> &box_::ed25519_box::SecretKey {
        &self.box_key_pair.secret_key
    }
}
