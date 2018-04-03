pub use sigchain_core::dashboard_protocol::*;
use sigchain_core::protocol::billing::*;
pub use sigchain_core::protocol::logs;
use yew::services::Task;

mod event;
pub use self::event::*;

pub struct Model {
    pub fetching: bool,
    pub show_cover_splash: bool,
    pub krypton_request_state: KryptonAppRequestState,
    pub pending_job: Option<Box<Task>>,
    pub is_data_fresh: bool,
    pub all_new_logs_loaded: bool,
    pub logs_page_limit: usize,

    pub session_is_expired: bool,

    pub me: Option<Identity>,
    pub team_name: String,
    pub members: Vec<TeamMember>,
    pub temporary_approval_seconds: Option<i64>,
    pub audit_logging_enabled: bool,
    pub billing_data: BillingData,

    pub selected_page: Page,

    pub team_page: TeamPage,
    pub hosts_page: HostsPage,
    pub logs_page: LogsPage,
    pub settings_page: SettingsPage,
}

#[derive(PartialEq, Clone)]
pub enum Page {
    Team,
    Hosts,
    AuditLogs,
    Settings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostSidebarTab {
    People,
    Logs,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum KryptonAppRequestState {
    Loading,
    LoadingResult,
    WaitingForResponse,
    Approved,
    Rejected,
    Error(String),
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamPage {
    pub role_filter: RoleFilter,
    pub last_access_filter: LastAccessFilter,
    pub search_query: String,
    pub selected_member: Option<TeamMember>,
    pub invite_link: Option<String>,
    pub handles_request_state: bool,
    pub show_removed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostsPage {
    pub hosts:Vec<Host>,
    pub search_query: String,
    pub selected_host: Option<Host>,
    pub selected_sidebar_tab: HostSidebarTab,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettingsPage {
    pub is_editing: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogsPage {
    pub logs:Vec<LogByUser>,
    pub search_query: String,
    pub log_chart_values: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Host {
    pub domain: String,
    pub people: Vec<TeamMemberForHost>,
    pub logs: Vec<LogByUser>,
    pub last_access_unix_seconds: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamMemberForHost {
    pub member: Identity,
    pub last_access_unix_seconds: i64,
    pub num_accesses: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogByUser {
    pub log:logs::Log,
    pub member_email:String,
}

use http;
use http::request::Request;

// Helper functions.
impl Model {
    pub fn empty() -> Model {
        Model {
            fetching: false,
            show_cover_splash: true,
            krypton_request_state: KryptonAppRequestState::None,
            members: vec![],
            is_data_fresh: false,
            all_new_logs_loaded: false,
            logs_page_limit: 1,

            session_is_expired: false,

            team_page: TeamPage {
                role_filter: RoleFilter::All,
                last_access_filter: LastAccessFilter::Any,
                search_query: "".into(),
                selected_member: None,
                invite_link: None,
                handles_request_state: false,
                show_removed: false,
            },
            me: None,
            team_name: "".into(),
            temporary_approval_seconds: None,
            audit_logging_enabled: false,
            billing_data: BillingData {
                billing_info: BillingInfo {
                    current_tier: PaymentTier {
                    name: "Starter".into(),
                        price: 0,
                        limit: BillingUsage {
                            members: 10,
                            hosts: 5,
                            logs_last_30_days: 1000,
                        }
                    },
                    usage: BillingUsage {
                            members: 0,
                            hosts: 1,
                            logs_last_30_days: 3,
                    },
                },
                url: "https://www-dev.krypt.co/billing".into(),
            },

            pending_job: None,

            selected_page: Page::Team,

            hosts_page: HostsPage{
                hosts: vec![],
                search_query: "".into(),
                selected_host: None,
                selected_sidebar_tab: HostSidebarTab::People,
            },
            logs_page: LogsPage { logs: vec![], search_query: "".into(), log_chart_values: vec![] },
            settings_page: SettingsPage{ is_editing: false },
        }
    }

    pub fn cancel_task_if_present(&mut self) {
        if let Some(mut task) = self.pending_job.take() {
            task.cancel();
        }

        self.pending_job = None;
    }

    fn add_auth_token<'a>(&self, request: &'a mut http::request::Builder) -> &'a mut http::request::Builder {
        use stdweb::unstable::TryInto;
        let url_fragment : Option<String> = js! {
            return window.location.hash;
        }.try_into().ok();
        if let Some(ref token) = url_fragment {
            request.header("X-Krypton-Dashboard-Token", token.trim_left_matches('#'));
        }
        request
    }

    pub fn build_get(&self, path: &str) -> http::request::Builder {
        let mut request = Request::get(path);
        self.add_auth_token(&mut request);
        request
    }

    pub fn build_post(&self, path: &str) -> http::request::Builder {
        let mut request = Request::post(path);
        self.add_auth_token(&mut request);
        request
    }
}
