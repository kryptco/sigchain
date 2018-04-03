extern crate time;

#[cfg(feature = "db")]
use db;
use b64data;
use super::{Result, SignedMessage, UTCSeconds};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Identity {
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub encryption_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub ssh_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub pgp_public_key: Vec<u8>,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MainChain {
    Create(GenesisBlock),
    Read(ReadBlocksRequest),
    Append(Block),
}


impl MainChain {
    pub fn last_block_hash(&self) -> Option<Vec<u8>> {
        use MainChain::*;
        match self {
            &Read(ref read_block) => {
                use TeamPointer::*;
                match read_block.team_pointer {
                    PublicKey(_) => None,
                    LastBlockHash(ref last_block_hash) => {
                        Some(last_block_hash.clone())
                    }
                }
            }
            &Append(ref write_block) => {
                Some(write_block.last_block_hash.clone())
            }
            &Create(_) => {
                None
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenesisBlock {
    pub team_info: TeamInfo,
    pub creator_identity: Identity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TeamPointer {
    PublicKey(
        #[serde(with = "b64data")]
        Vec<u8>),
    LastBlockHash(
        #[serde(with = "b64data")]
        Vec<u8>),
}

pub type SignedReadToken = SignedMessage;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadBlocksRequest {
    pub team_pointer: TeamPointer,
    #[serde(with = "b64data")]
    pub nonce: Vec<u8>,
    pub token: Option<SignedReadToken>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Response<T> {
    Success(T),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadBlocksResponse {
    pub blocks: Vec<SignedMessage>,
    pub more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyResponse {}

pub type E = EmptyResponse;

impl ReadBlocksResponse {
    #[cfg(feature = "db")]
    pub fn from_blocks(blocks: &[db::Block], more: bool) -> Result<ReadBlocksResponse> {
        Ok(ReadBlocksResponse{
            blocks: blocks.iter().map(|b| SignedMessage {
                public_key: b.member_public_key.clone(),
                message: b.operation.clone(),
                signature: b.signature.clone(),
            }).collect(),
            more,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    #[serde(with = "b64data")]
    pub last_block_hash: Vec<u8>,
    pub operation: Operation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Invite(Invitation),
    CloseInvitations(E),
    AcceptInvite(Identity),
    Remove(
        #[serde(with = "b64data")]
        Vec<u8>),
    Leave(E),
    SetPolicy(Policy),
    SetTeamInfo(TeamInfo),
    PinHostKey(SSHHostKey),
    UnpinHostKey(SSHHostKey),
    Promote(
        #[serde(with = "b64data")]
        Vec<u8>),
    Demote(
        #[serde(with = "b64data")]
        Vec<u8>),
    AddLoggingEndpoint(LoggingEndpoint),
    RemoveLoggingEndpoint(LoggingEndpoint),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Invitation {
    Direct(DirectInvitation),
    Indirect(IndirectInvitation),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndirectInvitationSecret {
    #[serde(with = "b64data")]
    pub initial_team_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub last_block_hash: Vec<u8>,
    #[serde(with = "b64data")]
    pub nonce_keypair_seed: Vec<u8>,
    pub restriction: IndirectInvitationRestriction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectInvitation {
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndirectInvitation {
    #[serde(with = "b64data")]
    pub nonce_public_key: Vec<u8>,
    pub restriction: IndirectInvitationRestriction,
    #[serde(with = "b64data")]
    pub invite_symmetric_key_hash: Vec<u8>,
    #[serde(with = "b64data")]
    pub invite_ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IndirectInvitationRestriction {
    Domain(String), // i.e a link that restricts joining member's email invitations to '@acme.co'.
    Emails(Vec<String>),
}

impl IndirectInvitation {
    #[cfg(feature = "crypto")]
    pub fn create_link(nonce_public_key: Vec<u8>, invite: IndirectInvitationSecret) -> Result<(IndirectInvitation, String)> {
        use base64;
        use serde_json;
        use crypto;
        use sha256;

        let plaintext = serde_json::to_vec(&invite)?;
        let ciphertext = crypto::secretbox::ephemeral_encrypt(plaintext);

        let link = format!("krypton://join_team/{}",
                           base64::encode_config(&ciphertext.symmetric_key, base64::URL_SAFE));

        let membership_invitation = IndirectInvitation {
            nonce_public_key,
            restriction: invite.restriction.clone(),
            invite_symmetric_key_hash: sha256::hash(&ciphertext.symmetric_key).0.as_ref().into(),
            invite_ciphertext: ciphertext.nonce_and_ciphertext,
        };
        Ok((membership_invitation, link))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Policy {
    #[serde(skip_serializing_if="Option::is_none")]
    pub temporary_approval_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamInfo {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SSHHostKey {
    pub host: String,
    #[serde(with = "b64data")]
    pub public_key: Vec<u8>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LoggingEndpoint {
    CommandEncrypted(E),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReadToken {
    Time(TimeToken),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeToken {
    #[serde(with = "b64data")]
    pub reader_public_key: Vec<u8>,
    pub expiration: UTCSeconds, // expiration of read permissions
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InviteSymmetricKeyHash {
    #[serde(with = "super::b64data")]
    pub symmetric_key_hash: Vec<u8>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InviteSymmetricKeyHashResponse {
    #[serde(with = "super::b64data")]
    pub ciphertext: Vec<u8>,
}
