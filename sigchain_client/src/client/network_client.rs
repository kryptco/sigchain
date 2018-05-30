extern crate reqwest;
use super::*;

use env;
use errors::Error;
use dotenv;

#[allow(dead_code)]
pub struct NetworkClient {
    pub key_pair: SignKeyPair,
    pub box_key_pair: box_::BoxKeyPair,
    pub server_endpoints: enclave_protocol::ServerEndpoints,
    pub team_public_key_and_checkpoint: enclave_protocol::TeamCheckpoint,
    pub db_connection: DBConnection,
}

impl NetworkClient {
    pub fn from_keypair_and_team_checkpoint(
        public_key_and_checkpoint: enclave_protocol::TeamCheckpoint,
        key_pair: SignKeyPair,
        box_key_pair: BoxKeyPair,
        file_name: &str,
        ) -> Result<NetworkClient> {
        Ok(NetworkClient {
            key_pair,
            box_key_pair,
            server_endpoints: public_key_and_checkpoint.server_endpoints.clone(),
            team_public_key_and_checkpoint: public_key_and_checkpoint,
            db_connection: NetworkClient::make_db_conn_to_file(file_name)?
        })
    }
    pub fn make_db_conn_to_file(file_name: &str) -> Result<DBConnection> {
        let conn = DBConnection::establish(file_name)?;
        db::run_migrations(&conn)?;
        Ok(conn)
    }
    pub fn make_db_conn() -> Result<DBConnection> {
        time_fn!("db_conn");
        dotenv().ok();

        let database_url = env::var("NETWORK_CLIENT_DATABASE_URL").or_else::<Error, _>(|_| {
            let mut home = env::var("HOME")?;
            home.push_str("/.kr/team.db");
            Ok(home.into())
        })?;
        let conn = DBConnection::establish(&database_url)?;

        db::run_migrations(&conn)?;
        Ok(conn)
    }
}

fn string_not_empty(s: String) -> Result<String> {
    if s == "" {
        Err("empty env var".into())
    } else {
        Ok(s)
    }
}

fn string_to_url_default_http(s: String) -> Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&s).or(reqwest::Url::parse(&("http://".to_string() + &s)))?)
}

fn nonempty_env(k: &str) -> Result<String> {
    Ok(env::var(k).map_err(Error::from).and_then(string_not_empty)?)
}

fn env_to_proxy_url(k: &str) -> Result<reqwest::Url> {
    nonempty_env(k).and_then(string_to_url_default_http)
}

fn new_proxy_aware_http_client() -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()?;

    let https_proxy = env_to_proxy_url("HTTPS_PROXY")
        .or_else(|_| env_to_proxy_url("https_proxy"))
        .or_else(|_| env_to_proxy_url("HTTP_PROXY"))
        .or_else(|_| env_to_proxy_url("http_proxy"))
        .ok();

    let http_proxy = env_to_proxy_url("HTTP_PROXY")
        .or_else(|_| env_to_proxy_url("http_proxy"))
        .ok();

    let no_proxy = nonempty_env("no_proxy").ok();

    builder.proxy(reqwest::Proxy::custom(move |url| {
        match (no_proxy.clone(), url.host_str().clone()) {
            (Some(no_proxy), Some(host)) => {
                if no_proxy.split(",").collect::<Vec<_>>().contains(&host) {
                    return None;
                }
            }
            _ => {}
        };

        match url.scheme() {
            "http" => http_proxy.clone(),
            "https" => https_proxy.clone().or(http_proxy.clone()),
            _ => None,
        }
    }));

    Ok(builder.build()?)
}

use ServerEndpoints;
pub fn default_broadcast<T>(server_endpoints: &ServerEndpoints, endpoint: &Endpoint, request: &SignedMessage) -> super::Result<T>
where T: serde::de::DeserializeOwned {
    use std::io::Read;
    let request_body = serde_json::to_vec(request)?;
    time_fn!("request");
    let client = new_proxy_aware_http_client()?;

    let mut response_bytes = client.put(
        &server_endpoints.url(endpoint),
    )?.body(request_body).send()?;
    let mut response_str = String::new();
    response_bytes.read_to_string(&mut response_str)?;

    use team::Response;
    match serde_json::from_str(&response_str) {
        Ok(Response::Success(t)) => Ok(t),
        Ok(Response::Error(s)) => bail!(s),
        Err(e) => bail!("Could not read json response\nError: {:?}\nResponse string: {:?}", e, response_str),
    }
}

impl traits::Broadcast for NetworkClient {
    fn broadcast<'a, T>(&self, endpoint: &Endpoint, request: &SignedMessage) -> super::Result<T>
        where T: serde::de::DeserializeOwned {
        default_broadcast(&self.team_public_key_and_checkpoint.server_endpoints, endpoint, request)
    }
}

impl traits::DBConnect for NetworkClient {
    fn db_conn(&self) -> &DBConnection {
        &self.db_connection
    }
}

impl traits::Identify for NetworkClient {
    fn identity_pk(&self) -> &[u8] {
        self.key_pair.public_key_bytes()
    }
    fn team_pk(&self) -> &[u8] {
        &self.team_public_key_and_checkpoint.team_public_key
    }
}

impl OwnedKeyPair for NetworkClient {
    fn commit_send<R: serde::de::DeserializeOwned>(&self, endpoint: &Endpoint, signed_message: &SignedMessage) -> Result<R> {
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
