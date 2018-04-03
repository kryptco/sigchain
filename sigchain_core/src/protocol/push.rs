#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PushSubscription {
    pub team_pointer: super::TeamPointer,
    pub action: PushSubscriptionAction,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PushSubscriptionAction {
    Subscribe(PushDevice),
    Unsubscribe(PushDevice),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PushDevice {
    Ios(String),
    Android(String),
    Queue(String),
}
