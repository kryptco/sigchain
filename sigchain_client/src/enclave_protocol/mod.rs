extern crate semver;
use self::semver::Version;

use {base64, b64data, time, crypto, serde_json, team, logging, protocol};
use std::result::Result as StdResult;

lazy_static! {
    pub static ref ENCLAVE_PROTOCOL_VERSION: Version = Version::new(2, 4, 0);
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: String,
    unix_seconds: u64,
    send_ack: bool,
    version: Version,
    body: RequestBody,
}

impl Request {
    pub fn new(body: RequestBody) -> super::Result<Request> {
        Ok(Request {
            id: base64::encode_config(&crypto::random_nonce()?, base64::URL_SAFE),
            unix_seconds: time::now().to_timespec().sec as u64,
            body,
            send_ack: true,
            version: ENCLAVE_PROTOCOL_VERSION.clone(),
        })
    }
}

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
        where S: Serializer {
        let mut state = serializer.serialize_struct("Request", 5)?;
        state.serialize_field("request_id", &self.id)?;
        state.serialize_field("unix_seconds", &self.unix_seconds)?;
        state.serialize_field("a", &self.send_ack)?;
        state.serialize_field("v", &self.version)?;
        use RequestBody::*;
        match self.body {
            MeRequest(ref me_request) =>
                state.serialize_field("me_request", me_request),
            ReadTeamRequest(ref read_team_request) =>
                state.serialize_field("read_team_request", read_team_request),
            TeamOperationRequest(ref team_operation_request) =>
                state.serialize_field("team_operation_request", team_operation_request),
            LogDecryptionRequest(ref log_decryption_request) =>
                state.serialize_field("log_decryption_request", log_decryption_request),
        }?;
        state.end()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RequestHeader {
    #[serde(rename = "request_id")]
    id: String,
    unix_seconds: u64,
    #[serde(rename = "a")]
    send_ack: bool,
    #[serde(rename = "v")]
    version: Version,
}

impl Request {
    pub fn from_json(json: &[u8]) -> super::Result<Request> {
        let header : RequestHeader = serde_json::from_slice(json)?;
        let body : RequestBody = serde_json::from_slice(json)?;
        Ok(Request{
            id: header.id,
            unix_seconds: header.unix_seconds,
            send_ack: header.send_ack,
            version: header.version,
            body,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Result<T> {
    Success(T),
    Error(Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub error: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RequestBody {
    MeRequest(MeRequest),
    ReadTeamRequest(ReadTeamRequest),
    TeamOperationRequest(TeamOperationRequest),
    LogDecryptionRequest(LogDecryptionRequest),
}

#[derive(Debug, Clone)]
pub struct Response {
    request_id: String,
    sns_endpoint_arn: Option<String>,
    version: Option<String>,
    approved_until: Option<u64>,
    tracking_id: Option<String>, // optional analytics tracking_id
    pub body: ResponseBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseBody {
    MeResponse(Result<MeResponse>),

    ReadTeamResponse(Result<team::SignedReadToken>),
    TeamOperationResponse(Result<TeamOperationResponse>),
    LogDecryptionResponse(Result<LogDecryptionResponse>),
}

impl <T> Result<T> {
    fn error(&self) -> Option<Error> {
        match self {
            &Result::Success(_) => { return None }
            &Result::Error(ref error) => { return Some(error.clone()) }
        }
    }
}
impl ResponseBody {
    pub fn error(&self) -> Option<Error> {
        let result = match self {
            &ResponseBody::MeResponse(ref result) => {
                result.error()
            }
            &ResponseBody::ReadTeamResponse(ref result) => {
                result.error()
            }
            &ResponseBody::TeamOperationResponse(ref result) => {
                result.error()
            }
            &ResponseBody::LogDecryptionResponse(ref result) => {
                result.error()
            }
        };

        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ResponseDe {
    request_id: String,
    sns_endpoint_arn: Option<String>,
    #[serde(rename = "v")]
    version: Option<String>,
    approved_until: Option<u64>,
    tracking_id: Option<String>, // optional analytics tracking_id

    me_response: Option<Result<MeResponse>>,

    create_team_response: Option<Result<TeamCheckpoint>>,
    read_team_response: Option<Result<team::SignedReadToken>>,
    team_operation_response: Option<Result<TeamOperationResponse>>,
    log_decryption_response: Option<Result<LogDecryptionResponse>>,
}

use serde::de::{Deserialize, Deserializer};
impl<'de> Deserialize<'de> for Response{
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
        where D: Deserializer<'de> {
        use serde::de::Error;
        let response_de = ResponseDe::deserialize(deserializer)?;
        let mut body = None;
        use ResponseBody::*;
        if let Some(r) = response_de.me_response {
            if body.is_some() {
                return Err(Error::custom("more than one enum variant present"));
            }
            body = Some(MeResponse(r));
        }
        if let Some(r) = response_de.read_team_response {
            if body.is_some() {
                return Err(Error::custom("more than one enum variant present"));
            }
            body = Some(ReadTeamResponse(r));
        }
        if let Some(r) = response_de.team_operation_response {
            if body.is_some() {
                return Err(Error::custom("more than one enum variant present"));
            }
            body = Some(TeamOperationResponse(r));
        }
        if let Some(r) = response_de.log_decryption_response {
            if body.is_some() {
                return Err(Error::custom("more than one enum variant present"));
            }
            body = Some(LogDecryptionResponse(r));
        }
        Ok(Response {
            request_id: response_de.request_id,
            sns_endpoint_arn: response_de.sns_endpoint_arn,
            version: response_de.version,
            approved_until: response_de.approved_until,
            tracking_id: response_de.tracking_id, // optional analytics tracking_id
            body: body.ok_or(Error::custom("no body enum variant present"))?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeRequest {
    pub pgp_user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    #[serde(rename = "public_key_wire", with = "b64data")]
    pub ssh_wire_public_key: Vec<u8>,
    pub email: String,
    #[serde(rename = "pgp_pk", with = "b64data::option", default)]
    pub pgp_public_key: Option<Vec<u8>>,
    pub team_checkpoint: Option<TeamCheckpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeResponse {
    pub me: Profile,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerEndpoints {
    pub api_host: String,
    pub billing_host: String,
}
impl ServerEndpoints {
    fn format_url(host: &str, endpoint: Option<&str>) -> String {
        let path = match endpoint {
            Some(path) => format!("/{}", path),
            None => "".into()
        };
        format!("https://{}{}", host, path).into()
    }
    pub fn prod() -> Self {
        ServerEndpoints {
            api_host: "api.krypt.co".into(),
            billing_host: "www.krypt.co".into(),
        }
    }
    pub fn dev() -> Self {
        ServerEndpoints {
            api_host: "api-dev.krypt.co".into(),
            billing_host: "www-dev.krypt.co".into(),
        }
    }
    pub fn staging() -> Self {
        ServerEndpoints {
            api_host: "api-staging.krypt.co".into(),
            billing_host: "www-staging.krypt.co".into(),
        }
    }
    pub fn url(&self, endpoint: &protocol::Endpoint)  -> String {
        use Endpoint::*;
        let path = match endpoint {
            &Sigchain => "sig_chain",
            &ChallengeEmail => "send_email_challenge",
            &VerifyEmail => "verify_email",
            &PushSubscription => "push_subscription",
            &InviteLinkCiphertext => "invite_link_ciphertext",
            &BillingInfo => "billing_info",
        };
        Self::format_url(&self.api_host, Some(path))
    }

    pub fn billing_url(&self, team_name: &str, team_public_key: &[u8], admin_public_key: &[u8], admin_email: &str) -> ::Result<String> {
        use {url, base64};
        let mut base_url = url::Url::parse(&Self::format_url(&self.billing_host, Some("billing/")))?;
        base_url.query_pairs_mut()
            .append_pair("tn", team_name)
            .append_pair("tid", &base64::encode_config(team_public_key, base64::URL_SAFE))
            .append_pair("aid", &base64::encode_config(admin_public_key, base64::URL_SAFE))
            .append_pair("aem", admin_email);
        Ok(base_url.into_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamCheckpoint {
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub team_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub last_block_hash: Vec<u8>,
    pub server_endpoints: ServerEndpoints,
}

impl TeamCheckpoint {
    pub fn sigchain_url(&self) -> String {
        self.server_endpoints.url(&protocol::Endpoint::Sigchain)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTeamRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadTeamRequest {
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamOperationRequest {
    pub operation: RequestableTeamOperation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamOperationResponse {
    #[serde(with = "b64data")]
    pub posted_block_hash: Vec<u8>,
    pub data: Option<TeamOperationResponseData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TeamOperationResponseData {
    InviteLink(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RequestableTeamOperation {
    DirectInvite(team::DirectInvitation),
    IndirectInvite(team::IndirectInvitationRestriction),
    CloseInvitations(team::E),

    SetPolicy(team::Policy),
    SetTeamInfo(team::TeamInfo),

    PinHostKey(team::SSHHostKey),
    UnpinHostKey(team::SSHHostKey),

    AddLoggingEndpoint(team::LoggingEndpoint),
    RemoveLoggingEndpoint(team::LoggingEndpoint),

    Promote(
        #[serde(with = "b64data")]
        Vec<u8>),
    Demote(
        #[serde(with = "b64data")]
        Vec<u8>),
    Remove(
        #[serde(with = "b64data")]
        Vec<u8>),
    Leave(team::E),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogDecryptionRequest {
    pub wrapped_key: logging::BoxedMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogDecryptionResponse {
    #[serde(with = "b64data")]
    pub log_decryption_key: Vec<u8>,
}
