use b64data;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailChallenge {
    #[serde(with = "b64data")]
    pub nonce: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct PutSendEmailChallengeRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct PutSendEmailChallengeResponse {
    pub error: Option<String>,
}
