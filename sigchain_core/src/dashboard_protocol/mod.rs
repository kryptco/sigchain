use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identity {
    pub email: String,
    #[serde(with = "super::b64data")]
    pub public_key: Vec<u8>,
    pub pgp_public_key: String,
    pub ssh_public_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostAccess {
    pub host: String,
    pub accesses: i64,
    pub last_access_unix_seconds: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamMember {
    pub identity: Identity,
    pub is_admin: bool,
    pub is_removed: bool,
    pub last_access: Option<logs::Log>,
    pub logins_today: i64,
    pub last_24_hours_accesses: Vec<logs::Log>,
    pub hosts: Vec<HostAccess>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KryptonStatus {
    NeedsApproval,
    Approved,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogsResponse {
    pub logs: Vec<logs::Log>,
}

#[derive(Deserialize, Serialize)]
pub struct PublicKeyRequest {
    #[serde(with = "super::b64data")]
    pub public_key: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct LinkResponse {
    pub link: String,
}

/** Dashboard request / response **/
pub type SearchQueryFilter = Option<String>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardRequest {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DashboardResult {
    Success(DashboardResponse),
    Error(DashboardError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DashboardError {
    NoTeam,
    Unknown(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardResponse {
    pub team_name: String,
    pub me: Identity,
    pub team_members: Vec<TeamMember>,
    pub temporary_approval_seconds: Option<i64>,
    pub audit_logging_enabled: bool,
    pub billing_data: BillingData,
    pub data_is_fresh: bool,
    pub all_new_logs_loaded: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillingData {
    pub billing_info: protocol::billing::BillingInfo,
    pub url: String,
}

impl BillingData {

    pub fn members_close_to_limit(&self) -> bool {
        BillingData::is_close(self.billing_info.usage.members,
                              self.billing_info.current_tier.limit.clone().map(|l| l.members).unwrap_or(None))
    }

    pub fn hosts_close_to_limit(&self) -> bool {
        BillingData::is_close(self.billing_info.usage.hosts,
                              self.billing_info.current_tier.limit.clone().map(|l| l.hosts).unwrap_or(None))
    }

    pub fn logs_close_to_limit(&self) -> bool {
        BillingData::is_close(self.billing_info.usage.logs_last_30_days,
                              self.billing_info.current_tier.limit.clone().map(|l| l.logs_last_30_days).unwrap_or(None))
    }

    fn is_close(usage:u64, limit: Option<u64>) -> bool {
        let limit = match limit {
            Some(l) => l,
            None => return false,
        };

        usage > limit/2
    }
}

impl BillingData {
    pub fn is_paid(&self) -> bool {
        self.billing_info.current_tier.price > 0
    }
}

// Filters
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RoleFilter {
    All,
    Admin,
    Removed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LastAccessFilter {
    Any,
    Host(String),
    Git,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TimeFilter {
    Last30Days,
    All,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogTypeFilter {
    SSH,
    Git,
    All,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogUserFilter {
    Admin,
    Any,
}

// helper methods

impl TeamMember {
    pub fn get_user_and_mail_domain(&self) -> (Option<&str>, Option<&str>) {
        self.identity.get_user_and_mail_domain()
    }
}

impl Identity {
    pub fn get_user_and_mail_domain(&self) -> (Option<&str>, Option<&str>) {
        let mut tokens = self.email.split("@");
        (tokens.next(), tokens.next())
    }
}

impl logs::LogBody {
    pub fn log_description(&self) -> String {
        match self {
            &logs::LogBody::Ssh(ref signature) => {
                let host = signature.host_authorization.as_ref().map(|h| h.host.clone()).unwrap_or("unknown host".into());
                return format!("ssh {} @ {}", signature.user, host);
            },
            &logs::LogBody::GitCommit(ref commit) => {
                let message = &commit.message_string.clone().unwrap_or("unknown".into());
                return format!("git commit -m \"{}\"", message);
            },
            &logs::LogBody::GitTag(ref tag) => {
                let tag_string = &tag.tag;
                let message = &tag.message_string.clone().unwrap_or("unknown".into());

                return format!("git tag {} -m \"{}\"", tag_string, message);
            }
        }
    }
}
