use errors::Result;
use b64data;
#[cfg(feature = "db")]
use db;
use team::{TeamPointer, SignedReadToken};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LogChain {
    Create(GenesisLogBlock),
    Read(ReadLogBlocksRequest),
    Append(LogBlock),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadLogBlocksRequest {
    #[serde(with = "b64data")]
    pub nonce: Vec<u8>,
    pub filter: LogFilter,
    pub token: Option<SignedReadToken>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LogFilter {
    Member(LogChainPointer),
    Team(TeamLogFilter), // logical log chain timestamp for team, starts at 0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LogChainPointer {
    GenesisBlock(LogChainGenesisPointer),
    LastBlockHash(
        #[serde(with = "b64data")]
        Vec<u8>
        ),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogChainGenesisPointer {
    #[serde(with = "b64data")]
    pub team_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub member_public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamLogFilter {
    #[serde(with = "b64data")]
    pub team_public_key: Vec<u8>,
    pub last_logical_timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxedMessage {
    #[serde(with = "b64data")]
    pub recipient_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub sender_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    pub ciphertext: Vec<u8>, // box(..., JSON(PlaintextBody))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WrappedKey {
    #[serde(with = "b64data")]
    pub recipient_public_key: Vec<u8>,
    // sender_public_key inferred by log chain writer
    #[serde(with = "b64data")]
    pub ciphertext: Vec<u8>, // box(..., JSON(PlaintextBody))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlaintextBody {
    LogEncryptionKey(
        #[serde(with = "b64data")]
        Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenesisLogBlock {
    pub team_pointer: TeamPointer,
    pub wrapped_keys: Vec<WrappedKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogBlock {
    #[serde(with = "b64data")]
    pub last_block_hash: Vec<u8>,
    pub operation: LogOperation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LogOperation {
    AddWrappedKeys(Vec<WrappedKey>),
    RotateKey(Vec<WrappedKey>),
    EncryptLog(EncryptedLog),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedLog {
    #[serde(with = "b64data")]
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadMemberLogBlocksResponse {
    pub blocks: Vec<super::SignedMessage>,
    pub more: bool,
}

impl ReadMemberLogBlocksResponse {
    #[cfg(feature = "db")]
    pub fn from_blocks(blocks: &[db::LogBlock], more: bool) -> Result<ReadMemberLogBlocksResponse> {
        Ok(ReadMemberLogBlocksResponse{
            blocks: blocks.iter().map(|b| super::SignedMessage {
                public_key: b.member_public_key.clone(),
                message: b.operation.clone(),
                signature: b.signature.clone(),
            }).collect(),
            more,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadTeamLogBlocksResponse {
    pub blocks: Vec<super::SignedMessage>,
    pub more: bool,
    pub update_logical_timestamp: Option<i64>,
}

impl ReadTeamLogBlocksResponse {
    #[cfg(feature = "db")]
    pub fn from_blocks(blocks: &[db::LogBlock], more: bool, update_logical_timestamp: Option<i64>) -> Result<ReadTeamLogBlocksResponse> {
        Ok(ReadTeamLogBlocksResponse{
            blocks: blocks.iter().map(|b| super::SignedMessage {
                public_key: b.member_public_key.clone(),
                message: b.operation.clone(),
                signature: b.signature.clone(),
            }).collect(),
            more,
            update_logical_timestamp,
        })
    }
}
