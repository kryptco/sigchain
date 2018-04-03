use b64data;

use team::SignedReadToken;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadBillingInfo {
    #[serde(with="b64data")]
    pub team_public_key: Vec<u8>,
    pub token: Option<SignedReadToken>,
}

type Cents = u64;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaymentTier {
    pub name: String,
    pub price: Cents,
    pub limit: BillingUsage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillingUsage {
    pub members: i64,
    pub hosts: i64,
    pub logs_last_30_days: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BillingInfo {
    pub current_tier: PaymentTier,
    pub usage: BillingUsage,
}

