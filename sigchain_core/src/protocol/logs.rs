use super::E;
use b64data;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Log {
    pub session: Session,
    pub unix_seconds: u64,
    pub body: LogBody,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub device_name: String,
    //  since the public key hash would give access to communication channels, we hash it again so the admin can still differentiate devices with the same name
    #[serde(with = "b64data")]
    pub workstation_public_key_double_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LogBody {
    Ssh(SSHSignature),
    GitTag(GitTagSignature),
    GitCommit(GitCommitSignature),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SSHSignature {
    pub user: String,
    pub host_authorization: Option<HostAuthorization>,
    #[serde(with = "b64data")]
    pub session_data: Vec<u8>,
    pub result: SSHSignatureResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SSHSignatureResult {
    UserRejected(E),
    HostMismatch(
        #[serde(with = "b64data::vec")]
        Vec<Vec<u8>>),
    Signature(
        #[serde(with = "b64data")]
        Vec<u8>
    ),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostAuthorization {
    pub host: String,
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
    #[serde(with = "b64data::option", default)]
    pub signature: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitCommitSignature {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    #[serde(with = "b64data")]
    pub message: Vec<u8>,
    pub message_string: Option<String>,
    pub result: GitSignatureResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitTagSignature {
    pub object: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub tag: String,
    pub tagger: String,
    #[serde(with = "b64data")]
    pub message: Vec<u8>,
    pub message_string: Option<String>,
    pub result: GitSignatureResult,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GitSignatureResult {
    UserRejected(E),
    Signature(
        #[serde(with = "b64data")]
        Vec<u8>),
    Error(String),
}

pub trait SignatureResult {
    fn is_success(&self) -> bool;
}

impl SignatureResult for LogBody {
    fn is_success(&self) -> bool {
        return match self {
            &LogBody::Ssh(ref ssh_signature) => {
                ssh_signature.result.is_success()
            }
            &LogBody::GitCommit(ref commit_signature) => {
                commit_signature.result.is_success()
            }
            &LogBody::GitTag(ref tag_signature) => {
                tag_signature.result.is_success()
            }
        };
    }
}

impl SignatureResult for SSHSignatureResult {
    fn is_success(&self) -> bool {
        return match self {
            &SSHSignatureResult::Signature(_) => { true }
            _ => { false }
        };
    }
}

impl SignatureResult for GitSignatureResult {
    fn is_success(&self) -> bool {
        return match self {
            &GitSignatureResult::Signature(_) => { true }
            _ => { false }
        };
    }
}
