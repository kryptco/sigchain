use super::*;
use team::*;

#[allow(dead_code)]
pub struct TestClient<'s> {
    pub sign_key_pair: SignKeyPair,
    pub box_key_pair: BoxKeyPair,
    pub team_public_key: Vec<u8>,
    pub db_connection: DBConnection,
    pub server_db_connection: &'s DBConnection,
}

impl<'s> TestClient<'s> {
    pub fn from_key_pair(sign_key_pair: SignKeyPair, box_key_pair: BoxKeyPair, team_public_key: Vec<u8>, client_db_url: &str, server_db_connection: &'s DBConnection) -> Result<TestClient<'s>> {
        Ok(
            TestClient {
                db_connection: TestClient::make_db_conn(client_db_url)?,
                server_db_connection,
                sign_key_pair,
                box_key_pair,
                team_public_key,
            }
        )
    }

    pub fn from_key_pair_temp_db(sign_key_pair: SignKeyPair, box_key_pair: BoxKeyPair, team_public_key: Vec<u8>, server_db_connection: &'s DBConnection) -> Result<TestClient<'s>> {
        Ok(
            TestClient {
                db_connection: TestClient::make_db_conn("")?,
                server_db_connection,
                sign_key_pair,
                box_key_pair,
                team_public_key,
            }
        )
    }

    fn get_server_db_connection(&self) -> Result<&DBConnection> {
        Ok(self.server_db_connection)
    }

    pub fn join_team_request(&self,
                             last_block_hash: Vec<u8>,
                             nonce_key_pair: &SignKeyPair,
                             identity: team::Identity,
    ) -> Result<SignedMessage> {
        while self.read_new_block_from_hash_with_key(None,&nonce_key_pair)?.more {}
        let write_block = Block {
            last_block_hash: last_block_hash.into(),
            operation: Operation::AcceptInvite(identity),
        };
        let body = Body::Main(MainChain::Append(write_block.clone()));
        self.prepare_payload_with_key_pair(body, nonce_key_pair)
    }

    fn make_db_conn(client_db_url: &str) -> Result<DBConnection> {
        time_fn!("db_conn");

        let conn = DBConnection::establish(client_db_url)?;

        db::run_migrations(&conn)?;
        Ok(conn)
    }

    fn prepare_payload_with_key_pair(&self, payload: Body, key_pair: &SignKeyPair) -> Result<SignedMessage> {
        let request = SignedMessage::from_message(Message::new(payload.clone()), key_pair)?;
        self.verified_payload_with_db_txn(&request)?;
        Ok(request)
    }
}

impl<'s> traits::Identify for TestClient<'s> {
    fn identity_pk(&self) -> &[u8] {
        self.sign_key_pair.public_key_bytes()
    }
    fn team_pk(&self) -> &[u8] {
        &self.team_public_key
    }
}

impl<'s> traits::DBConnect for TestClient<'s> {
    fn db_conn(&self) -> &DBConnection {
        &self.db_connection
    }
}

impl<'s> OwnedKeyPair for TestClient<'s> {
    fn commit_send<R: serde::de::DeserializeOwned>(&self, endpoint: &Endpoint, signed_message: &SignedMessage) -> Result<R> {
        use traits::{Broadcast, DBConnect};
        self.db_conn().transaction::<_, Error, _>(|| {
            self.verified_payload(signed_message)?;
            self.broadcast::<R>(endpoint, signed_message)
        })
    }
    fn sign_key_pair(&self) -> &SignKeyPair {
        &self.sign_key_pair
    }
    fn box_public_key(&self) -> &box_::ed25519_box::PublicKey {
        &self.box_key_pair.public_key
    }

    fn box_secret_key(&self) -> &box_::ed25519_box::SecretKey {
        &self.box_key_pair.secret_key
    }
}

impl<'s> traits::Broadcast for TestClient<'s> {
    fn broadcast<'a, T>(&self, _endpoint: &protocol::Endpoint, request: &SignedMessage) -> Result<T> where
        T: serde::de::DeserializeOwned {
        self.get_server_db_connection()?.transaction(|| {
            Ok(serde_json::from_str(
                &verify_and_process_request(self.get_server_db_connection()?, request)?.json_response_to_client
            )?)
        })
    }
}
