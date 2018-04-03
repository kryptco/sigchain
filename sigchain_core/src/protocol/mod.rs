extern crate semver;
use self::semver::Version;

pub mod logs;
pub use self::logs::*;

pub mod logging;
pub use self::logging::*;

pub mod email;
pub use self::email::*;

pub mod team;
pub use self::team::*;

pub mod push;
pub use self::push::*;

pub mod billing;

#[cfg(feature = "crypto")]
use crypto::*;
#[cfg(feature = "crypto")]
use sha256;

use b64data;
use serde_json;
use super::Result;

use chrono::offset::Utc;

pub const CLOCK_SKEW_THRESHOLD: i64 = 60*60;

// NOTE: Do not use other features of semantic versioning (i.e. pre and build).
// iOS code does not support it.
lazy_static! {
    pub static ref CURRENT_VERSION: Version = Version::new(1, 0, 0);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedMessage {
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
    pub message: String,
    #[serde(with = "b64data")]
    pub signature: Vec<u8>,
}

#[cfg(feature = "crypto")]
impl SignedMessage {
    pub fn from_message(payload: Message, key_pair: &SignKeyPair) -> Result<SignedMessage> {
        let payload_json = serde_json::to_string(&payload)?;
        let sig = ed25519::sign_detached(&payload_json.as_bytes(), &key_pair.secret_key);
        Ok(SignedMessage {
            public_key: key_pair.public_key_bytes().into(),
            message: payload_json,
            signature: sig.0.to_vec(),
        })
    }
    pub fn payload_hash(&self) -> Vec<u8> {
        let inner_hashes =
            vec![self.public_key.as_slice(), self.message.as_bytes()].into_iter()
            .map(sha256::hash)
            .flat_map(|h| h.0.to_vec())
            .collect::<Vec<u8>>();
        sha256::hash(&inner_hashes).0.to_vec()
    }
}

type UTCSeconds = i64;
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Header {
    pub utc_time: UTCSeconds,
    pub protocol_version: Version,
}

impl Header {
    pub fn new() -> Header {
        Header {
            utc_time: Utc::now().timestamp(),
            protocol_version: CURRENT_VERSION.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Main(MainChain),
    Log(LogChain),
    ReadToken(ReadToken),
    EmailChallenge(EmailChallenge),
    PushSubscription(PushSubscription),
    ReadBillingInfo(billing::ReadBillingInfo),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub body: Body,
}

impl Message {
    pub fn new(body: Body) -> Message {
        Message {
            header: Header::new(),
            body,
        }
    }
}

pub enum Endpoint {
    Sigchain,
    ChallengeEmail,
    VerifyEmail,
    PushSubscription,
    InviteLinkCiphertext,
    BillingInfo,
}
