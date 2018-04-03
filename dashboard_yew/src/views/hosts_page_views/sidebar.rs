use context::*;
use models::*;
use yew::prelude::*;
use sigchain_core::time_util::*;

fn view_for_access_log_by_member(log_by_user: &LogByUser) -> Html<Context, Model> {
    let mut tokens = log_by_user.member_email.split("@");
    let (user, domain) =  (tokens.next(), tokens.next());

    let log = log_by_user.log.clone();

    html! {
        <div class="item",>
            <div class="terminal",>
                <div>
                    <span class="terminal-cmd",>{ "$ " } </span>
                    { log.body.log_description() }
                </div>
                <div class="details",>
                    <div class="meta",>
                        <span class="white",>{user.unwrap_or("--")}</span>
                        {"@"}{domain.unwrap_or("--")}
                    </div>
                    <div class="meta",>
                        { log.unix_seconds.time_ago() }
                    </div>
                </div>
            </div>
        </div>
    }
}

fn view_for_sidebar_person_item(member:&TeamMemberForHost) -> Html<Context, Model> {
    let (user,domain) = member.member.get_user_and_mail_domain();

    html! {
            <div class="item",>
                <div>
                    <div class=("data", "host"),>
                        {user.unwrap_or("--")}
                        <span class="mail-domain",>
                            {"@"}{domain.unwrap_or("--")}
                        </span>
                    </div>
                    <div class="meta",> {"Last access "} { member.last_access_unix_seconds.time_ago() }</div>

                </div>
                <div class="right",>
                    <div><span class="data",>{member.num_accesses} </span> { " Accesses"}</div>
                    <div><a onclick=|_| Event::SelectPage(Page::AuditLogs),>{"View logs"}</a></div>
                </div>
            </div>
    }
}

impl Model {
    pub fn view_host_sidebar(&self) -> Html<Context, Model> {

        let host:Host;
        match self.hosts_page.selected_host.clone() {
            Some(the_host) => {
                host = the_host.clone();
            },
            _=> { return html! { <div id="sidebar",></div> }; }
        };

        let people_count = host.people.len();
        let logs_count = host.logs.len();

        // paginate the logs
        use constants;
        let log_items_to_show = constants::LOG_DOM_PAGE_SIZE * self.logs_page_limit;
        let has_more_logs = logs_count > log_items_to_show;

        let logs:Vec<&LogByUser>;
        if has_more_logs {
            logs = host.logs.iter().take(log_items_to_show).collect();
        } else {
            logs = host.logs.iter().collect();
        }
        let next_log_page = self.logs_page_limit + 1;

        html! {
            <div id="sidebar",>
                <div class="subhead",>
                    <img src="img/x.svg", class=("close", "icon"), onclick=|_| { js!{ closeSidebar(); }; Event::CloseHostSidebar },></img>
                    <div>
                        <span class="host-name",>
                            { host.domain }
                        </span>
                    </div>
                    <div class=("last-active", "meta"),>
                        {"Last accessed "}{ host.last_access_unix_seconds.time_ago() }
                    </div>
                </div> // subhead
                <div class="host-details",>
                    <ul class="menu",>
                        <li id="side_people-title", class=("side-menu",
                            match self.hosts_page.selected_sidebar_tab {
                                HostSidebarTab::People => "active",
                                _ => "",
                            }
                        ), onclick=|_| {js!{showSideTab("side_people");}; Event::Ignore},>
                            {"People"}
                        </li>
                        <li id="side_logs-title", class=("side-menu",
                            match self.hosts_page.selected_sidebar_tab {
                                HostSidebarTab::Logs => "active",
                                _ => "",
                            }
                        ), onclick=|_| {js!{showSideTab("side_logs");}; Event::Ignore},>
                            {"Audit Logs"}
                        </li>
                    </ul>
                    <div id="side_people", class=("tab",
                            match self.hosts_page.selected_sidebar_tab {
                                HostSidebarTab::People => "show",
                                _ => "",
                            }
                        ),>
                        <div class="side-table",>
                            <div class="item",>
                                <span>
                                    <span id="team-num", class="data",> { people_count } </span>
                                    {" team members"}
                                </span>
                                <span class="filter",>
                                    {"Sort by"}
                                    <select>
                                        <option>{"Accesses"}</option>
                                    </select>
                                </span>
                            </div>
                            <div class="table-data",>
                                {
                                    for host.people.iter().map(|p| view_for_sidebar_person_item(p) )
                                }
                            </div> //table-data
                        </div> //side-table

                    </div> //side_hosts

                    <div id="side_logs", class=("tab",
                            match self.hosts_page.selected_sidebar_tab {
                                HostSidebarTab::Logs => "show",
                                _ => "",
                            }
                        ),>
                        <div class="side-table",>
                            <div class="item",>
                                <span>
                                    <span id="logs-num", class="data",> { logs_count } </span>
                                    {" access logs"}
                                </span>
                                <a class="filter", onclick=|_|{ Event::SelectPage(Page::AuditLogs) },>
                                    {"View all logs"}
                                </a>
                            </div>
                            <div class="table-data",>
                                {
                                    for logs.iter().map(|l| view_for_access_log_by_member(l) )
                                }
                                // pagination
                                {
                                    if has_more_logs {
                                        html! {
                                            <button class="load-logs-button", onclick=move |_| Event::ViewNextLogsPage(next_log_page),>
                                                {"Load more logs"}
                                            </button>
                                        }
                                    } else {
                                        html! { <div> </div>}
                                    }
                                }
                            </div> //table-data
                        </div> //side-table

                    </div> //side_logs
                </div> //sidebar
        }
    }
}
