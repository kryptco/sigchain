use b64data;

use team::SignedReadToken;

pub const DEFAULT_UNIT_DESCRIPTION: &'static str = "per user";

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
    pub limit: Option<BillingLimit>,
    pub unit_description: String, // i.e. "per developer"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillingUsage {
    pub members: u64,
    pub hosts: u64,
    pub logs_last_30_days: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillingLimit {
    pub members: Option<u64>,
    pub hosts: Option<u64>,
    pub logs_last_30_days: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BillingInfo {
    pub current_tier: PaymentTier,
    pub usage: BillingUsage,
}

