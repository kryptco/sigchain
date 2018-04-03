use context::*;
use models::*;
use yew::prelude::*;
use super::member_item::*;
use sigchain_core::time_util::*;

pub fn view_for_access_log(log: &logs::Log) -> Html<Context, Model> {
    html! {
        <div class="item",>
            <div class="terminal",>
                <div>
                    <span class="terminal-cmd",>{ "$ " } </span>
                    { log.body.log_description() }
                </div>
                <div class="meta",>
                    {"Accessed: "} { log.unix_seconds.time_ago() }
                </div>
            </div>
        </div>
    }
}

pub fn view_host_access_item(host_access:&HostAccess) -> Html<Context, Model> {
    html! {
        <div class="item",>
            <div>
                <div class=("data", "host"),>
                    { host_access.host.clone() }
                </div>
                <div class="meta",>{"Last access: "} { host_access.last_access_unix_seconds.time_ago() }</div>
            </div>
            <div class="right",>
                <div><span class="data",> { host_access.accesses } </span> {" Accesses"} </div>
                <div><a onclick=|_| Event::SelectPage(Page::AuditLogs),>{"View logs"}</a></div>
            </div>
        </div>
    }
}


impl Model {
    pub fn view_member_sidebar(&self) -> Html<Context, Model> {

        match self.team_page.selected_member {
            Some(ref member) => {
                let (user, domain) = member.get_user_and_mail_domain();
                let is_me = self.me.clone().map(|m| m.public_key == member.identity.public_key).unwrap_or(false);

                // paginate the logs
                use constants;
                let log_items_to_show = constants::LOG_DOM_PAGE_SIZE * self.logs_page_limit;
                let has_more_logs = member.last_24_hours_accesses.len() > log_items_to_show;

                let logs:Vec<&logs::Log>;
                if has_more_logs {
                    logs = member.last_24_hours_accesses.iter().take(log_items_to_show).collect();
                } else {
                    logs = member.last_24_hours_accesses.iter().collect();
                }
                let next_log_page = self.logs_page_limit + 1;


                return html! {
                        <div id="sidebar",>
                            <div class="subhead",>
                                <img src="img/x.svg", class=("close", "icon"), onclick=|_| { js!{ closeSidebar(); }; Event::CloseTeamSidebar },></img>
                                <div>
                                    <span class="user-email",>
                                        {user.unwrap_or("--")} <span class="mail-domain",>{"@"}{domain.unwrap_or("--")}</span>
                                    </span>
                                    { view_for_is_admin(member) }
                                </div>
                                { view_for_last_active(member) }

                                <div>
                                    <div class="list-item-public-keys",>
                                        { view_for_copy_buttons(member) }
                                    </div>
                                    <div class="list-item-actions",>
                                        { view_for_remove_button(member, is_me) }
                                        { view_for_role_change_button(member) }
                                    </div>
                                </div>
                            </div> // subhead
                            <div class="user-details",>
                                <ul class="menu",>
                                    <li id="side_hosts-title", class=("side-menu", "active"), onclick=|_| {js!{showSideTab("side_hosts");}; Event::Ignore},>
                                        {"Accessed Hosts"}
                                    </li>
                                    <li id="side_logs-title", class="side-menu", onclick=|_| {js!{showSideTab("side_logs");}; Event::ViewNextLogsPage(1)},>
                                        {"Audit Logs"}
                                    </li>
                                </ul>
                                <div id="side_hosts", class=("tab", "show"),>
                                    <div class="side-table",>
                                        <div class="item",>
                                            <span>
                                                <span id="host-num", class="data",> { member.hosts.len() } </span>
                                                {" hosts accessed"}
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
                                                for member.hosts.iter().map(|h| view_host_access_item(h) )
                                            }
                                        </div> //table-data
                                    </div> //side-table

                                </div> //side_hosts

                                <div id="side_logs", class="tab",>
                                    <div class="side-table",>
                                        <div class="item",>
                                            <span>
                                                <span id="logs-num", class="data",> { member.last_24_hours_accesses.len() } </span>
                                                {" access logs"}
                                            </span>
                                            <a class="filter", onclick=|_| Event::SelectPage(Page::AuditLogs),>
                                                {"View all logs"}
                                            </a>
                                        </div>
                                        <div class="table-data",>
                                            {
                                                for logs.iter().map(|l| view_for_access_log(l) )
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
                };
            }
            _ => {
                return html! { <div id="sidebar",></div> };
            }
        }
    }
}


