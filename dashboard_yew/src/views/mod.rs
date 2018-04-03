mod loading_views;
mod team_page_views;
mod hosts_page_views;
mod audit_logs_views;
mod settings;

use models::*;
use yew::prelude::*;
use context::*;

fn view_for_nav_button(page: Page, selected:Page) -> Html<Context, Model> {
    let li_class:&str;
    if  page == selected {
        li_class = "selected";
    } else {
        li_class = "";
    }

    match page {
        Page::Team => {
            html!{
                <li class={ li_class },><a onclick=|_| Event::SelectPage(Page::Team) ,> {"Team"}</a></li>
            }
        }
        Page::Hosts => {
            html!{ <li class={ li_class },><a onclick=|_| Event::SelectPage(Page::Hosts) ,> {"Hosts"}</a></li> }
        }
        Page::AuditLogs => {
            html!{ <li class={ li_class },><a onclick=|_| Event::SelectPage(Page::AuditLogs) ,> {"Audit Logs"}</a></li> }
        }
        Page::Settings => {
            html!{ <li class={ li_class },><a onclick=|_| Event::SelectPage(Page::Settings) ,> {"Settings"}</a></li> }
        }
    }
}


impl Model {
    fn view_for_fresh_data(&self) -> Html<Context, Model> {
        if self.is_data_fresh {
            if !self.all_new_logs_loaded {
                return html! {
                    <div id="incoming-log-data-warning",>
                        <p class="incoming-log-data-text",>
                            { "Not all log data has loaded yet. Loading more in background..." }
                        </p>
                    </div>
                };
            }
            return html! {<div></div>}
        }

        html!{
            <div id="stale-data-warning",>
                <p class="stale-data-text",>
                    { "Could not load new data from the server. Please check your internet connection."}
                </p>
            </div>
        }
    }

    pub fn view_for_app(&self) -> Html<Context, Model> {
        // set the page class attribute based on splash status
        let page_class:&str;
        if  self.show_cover_splash {
            page_class = "blurry";
        } else {
            page_class = "";
        }

        let selected_page = self.selected_page.clone();

        html! {

           <div>

                // modals hidden in the background
                { self.view_for_request_loader() }
                { self.view_for_fresh_data() }
                { self.view_for_bar_loader() }
                { self.view_for_add_members() }

                // main page
                <div id="page", class={page_class},>
                    <div id="nav-bar",>
                        <div class="team-header",>
                            <img class="app-logo", src="img/krypton-shield.svg",/>
                            <p class="attr-value", id="team-name",> { self.team_name.clone() }</p>
                        </div>
                        <ul>
                            { view_for_nav_button(Page::Team, selected_page.clone()) }
                            { view_for_nav_button(Page::Hosts, selected_page.clone()) }
                            { view_for_nav_button(Page::AuditLogs, selected_page.clone()) }
                            { view_for_nav_button(Page::Settings, selected_page.clone()) }
                        </ul>
                    </div>
                    <div class="main-content",>
                            {
                                match self.selected_page {
                                    Page::Team => {
                                        self.view_for_team_page()
                                    }
                                    Page::Hosts => {
                                        self.view_for_hosts_page()
                                    }
                                    Page::AuditLogs => {
                                        self.view_for_logs_page()
                                    }
                                    Page::Settings => {
                                        self.view_for_settings_page()
                                    }
                                }
                            }
                    </div> // main-content
               </div> // page
          </div>
        }
    }
}
