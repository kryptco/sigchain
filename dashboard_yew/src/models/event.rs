pub use sigchain_core::dashboard_protocol::*;
pub use sigchain_core::protocol::logs;
use sigchain_core::protocol::team::IndirectInvitationRestriction;
use models::*;

pub type IsRecurring = bool;

#[derive(Clone)]
pub enum Event {
    // data
    DoRequest(IsRecurring),
    HandleResponse(DashboardResponse, IsRecurring),
    SessionExpired,

    // member actions
    Remove(TeamMember),
    Promote(TeamMember),
    Demote(TeamMember),

    // searching
    SearchChanged(String),

    // invite
    CreateInvite(IndirectInvitationRestriction),
    DidCreateInviteURL(String),
    ClearInviteURL,

    // sidebar
    OpenTeamSidebar(TeamMember),
    CloseTeamSidebar,

    OpenHostSidebar(Host, HostSidebarTab),
    CloseHostSidebar,

    // pagination
    SelectPage(Page),

    //Log Pagination
    ViewNextLogsPage(usize),

    // state help
    Initial,
    CheckApprovalStatus,
    EventAfter(u64, Vec<Event>),
    RequestStateChanged(KryptonAppRequestState),
    Many(Vec<Event>),
    Ignore,

    // helpers
    DrawChart,
    ToggleShowRemovedMembers,

    // settings
    SetTeamName(String),
    EditApprovalWindow(Option<i64>),
    EnableAuditLogging(bool),
    ToggleIsEditingSetting(bool),

}
