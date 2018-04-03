use context::*;
use models::*;
use yew::prelude::*;

mod filters;

mod host_item;
use self::host_item::*;

mod sidebar;

impl Model {
    pub fn view_for_hosts_page(&self) -> Html<Context, Model> {

        let hosts = self.hosts_page.hosts.clone();

        let hosts_num = hosts.len();
        let filtered_hosts:Vec<Host>;

        let search_query = self.hosts_page.search_query.clone();
        match search_query.is_empty() {
            false => {
                filtered_hosts = hosts.into_iter().filter(|h| h.domain.contains(&search_query)).collect();
            },
            _ => {
                filtered_hosts = hosts.clone();
            }
        };

        html! {
              <div>
                <div id="main-window",>
                      <div class="main-window-title",>
                          <span class="title",>{"Hosts"}</span>
                      </div>
                      <div id="num-members", class="main-window-list-title",>
                          { format!("{} SSH Hosts", hosts_num) }
                      </div>

                      <div class="filters",>
                        { self.view_for_hosts_page_filters() }
                      </div>

                      <div id="hosts-container", class="list-container",>
                          <div class="list-header-host",>
                            {"HOST"}
                          </div>
                          <div class="list-header-people",>
                                {"PEOPLE"}
                          </div>
                          <div class="list-header-accesses",>
                                {"ACCESSES"}
                          </div>
                          <div class="list-header-host-last-accessed",>
                                {"LAST ACCESSED"}
                          </div>
                          <div class="list-header-border",></div>

                            {
                                for filtered_hosts.iter().map(|h| view_for_host_item(h) )
                            }

                      </div> // list-container
                        {
                            match search_query.clone().is_empty() {
                                false => {
                                  html! {<div id="search-results-footer", style="grid-column: span 12; text-align: center; opacity: 0.5; font-style: italic;",>
                                            { format!("Showing {} out-of {} results for search \"{}\"", filtered_hosts.len(), hosts_num, search_query) }
                                        </div>}
                                },
                                true => {html! {<div></div>} }
                            }
                        }
                    // side bar
                    <div id="sidebar-overlay", onclick=|_| { js! { closeSidebar(); }; Event::Ignore },></div>
                    { self.view_host_sidebar() }
              </div> // main window
            </div> // root
        }
    }
}
