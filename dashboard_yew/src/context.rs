use yew::services::fetch::FetchService;
use yew::services::timeout::TimeoutService;
use yew::services::console::ConsoleService;

pub struct Context {
    pub web: FetchService,
    pub timeout: TimeoutService,
    pub console: ConsoleService,
}
