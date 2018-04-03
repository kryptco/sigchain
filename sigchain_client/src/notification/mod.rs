use db::TeamMembership;

pub struct NotificationsAndResponse {
    pub json_response_to_client: String,
    pub notification_actions: Vec<NotificationAction>,
}

impl NotificationsAndResponse {
    pub fn no_notifications(response: String) -> NotificationsAndResponse {
        NotificationsAndResponse {
            json_response_to_client: response,
            notification_actions: Vec::new(),
        }
    }
}

pub enum NotificationAction {
    TeamPush(Vec<u8>),
    Unsubscribe(TeamMembership),
}