#![recursion_limit="1024"]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate yew;

#[macro_use]
extern crate stdweb;

extern crate sigchain_core;
extern crate chrono;
extern crate serde_json;
extern crate time;
extern crate serde;

extern crate http;

use yew::prelude::*;
use yew::services::timeout::TimeoutService;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService};
use yew::html::*;
use stdweb::web::document;

mod context;
use context::*;

mod models;
use models::*;

mod views;
mod controllers;

mod constants {
    pub const LOG_DOM_PAGE_SIZE:usize = 100;
}

impl Model {
    fn view_for_expired_session(&self) -> Html<Context, Model> {
        html! {
            <div id="expired-session",>
                <p>{"This session has expired."}</p>
                <p> {"Please re-open the dashboard by running the command below."}</p>
                <div class="terminal",>
                    <span class="terminal-cmd",> {"$ "} </span>
                    { "kr team dashboard" }
                </div>
            </div>
       }
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        if self.session_is_expired {
            html!{ <div > { self.view_for_expired_session() } </div> }
        } else {
            html!{ <div> { self.view_for_app() } </div> }
        }
    }
}

fn mount_app(selector: &'static str, app: Scope<Context, Model>) {
    let element = document().query_selector(selector).unwrap();
    app.mount(element);
}

fn main() {
    yew::initialize();
    let context = Context {
        web: FetchService::new(),
        timeout: TimeoutService::new(),
        console: ConsoleService,
    };
    let mut app: App<Context, Model> = App::new(context);
    app.get_env().sender().send(ComponentUpdate::Message(Event::Initial));
    mount_app(".app", app);
    yew::run_loop();
}