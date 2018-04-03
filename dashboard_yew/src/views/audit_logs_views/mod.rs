use context::*;
use models::*;

use yew::prelude::*;

mod filters;

mod log_item;
use self::log_item::*;

use sigchain_core::protocol::logs::*;
use constants;

fn log_body_matches(log_body:&LogBody, search_query: &String) -> bool {
    return match log_body {
        &LogBody::Ssh(ref signature) => {
            let host:String = signature.clone().host_authorization.map(|h| h.host).unwrap_or("unknown host".into());
            host.contains(search_query) || signature.user.contains(search_query) || Into::<String>::into("ssh").contains(search_query)
        }
        &LogBody::GitCommit(ref ref_commit) => {
            let commit = ref_commit.clone();
            commit.author.contains(search_query)
                || commit.committer.contains(search_query)
                || commit.message_string.unwrap_or("".into()).contains(search_query)
                || Into::<String>::into("git commit").contains(search_query)
        }
        &LogBody::GitTag(ref ref_tag) => {
            let tag = ref_tag.clone();

            tag.tag.contains(search_query)
                || tag.tagger.contains(search_query)
                || tag.message_string.unwrap_or("".into()).contains(search_query)
                || tag.type_.contains(search_query)
                || Into::<String>::into("git tag").contains(search_query)
        }
    };
}
impl Model {
    pub fn view_for_logs_page(&self) -> Html<Context, Model> {

        let mut filtered_logs:Vec<&LogByUser>;

        let log_items_to_show = constants::LOG_DOM_PAGE_SIZE * self.logs_page_limit;

        // filter the logs
        let search_query = self.logs_page.search_query.clone();
        match search_query.is_empty() {
            false => {
                filtered_logs = self.logs_page.logs.iter().filter(|l| {
                    l.member_email.contains(&search_query) || log_body_matches(&l.log.body, &search_query)
                }).collect();
            },
            _ => {
                filtered_logs = self.logs_page.logs.iter().map(|l| l).collect();
            }
        };

        // paginate the logs
        let has_more_logs = filtered_logs.len() > log_items_to_show;
        if has_more_logs {
            filtered_logs = filtered_logs.into_iter().take(log_items_to_show).collect();
        }

        let next_log_page = self.logs_page_limit + 1;

        html! {
              <div>
                <div id="main-window",>
                      <div class="main-window-title",>
                          <span class="title",>{"Audit Logs"}</span>
                      </div>

                      <div class="analytics",>
                        <div class="card",>
                            <div class="graph-header",>
                                <p>
                                    {"# Krypton Requests / 30m"}
                                </p>
                            </div>
                            <div class="logs-chart", style="display:flex;",></div>
                            <script type="application/javascript", src="js/chart.init.js",></script>
                        </div>
                      </div>

                      <div id="logs-container", class="card",>
                          <div class="header",>
                            <span>{"Audit logs"}</span>
                            { self.view_for_logs_page_filters() }
                          </div>
                         <table>
                            <tr>
                                <th>{"Type"}</th>
                                <th>{""}</th>
                                <th>{"Info"}</th>
                                <th>{"User"}</th>
                                <th>{"Timestamp"}</th>
                            </tr>
                            {
                                for filtered_logs.iter().map(|l| view_for_logs_log_item(l) )
                            }
                         </table>

                      </div> // list-container
                        {
                            match search_query.is_empty() {
                                false => {
                                  html! {<div id="search-results-footer", style="padding-top: 10px; grid-column: span 12; text-align: center; opacity: 0.5; font-style: italic;",>
                                            { format!("Showing {} results for search \"{}\"", filtered_logs.len(), search_query) }
                                        </div>}
                                },
                                true => {html! {<div></div>} }
                            }
                        }
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
              </div> // main window
            </div> // root
        }
    }
}