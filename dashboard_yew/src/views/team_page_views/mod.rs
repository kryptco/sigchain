mod add_team_member;
mod copy_helper;
mod filters;
mod member_item;
mod sidebar;

use context::*;
use models::*;
use yew::prelude::*;
pub use self::filters::*;
pub use self::member_item::*;
pub use self::sidebar::*;

impl Model {
    pub fn view_for_team_page(&self) -> Html<Context, Model> {
        let num_members = self.members.clone().into_iter().filter(|m| !m.is_removed).count();
        let num_removed = self.members.clone().into_iter().filter(|m| m.is_removed).count();

        let me_clone = self.me.clone();

        let all_members:Vec<TeamMember> = match self.team_page.show_removed {
            true => { self.members.clone() }
            _=> { self.members.clone().into_iter().filter(|m| !m.is_removed).collect() }
        };

        let all_count = all_members.len();
        let filtered_members:Vec<TeamMember>;

        let search_query = self.team_page.search_query.clone();
        match search_query.is_empty() {
            false => {
                filtered_members = all_members.into_iter().filter(|m| m.identity.email.contains(&search_query)).collect();
            },
            _ => {
                filtered_members = all_members.clone();
            }
        };

        let num_members_html:Html<Context, Model> = match self.team_page.show_removed {
            true => {
                html! {
                    <div>
                        { format!("{} members (+{} removed)", num_members, num_removed) }
                        <a class="show-removed-button", onclick=|_| Event::ToggleShowRemovedMembers,> {"Hide Removed" } </a>
                    </div>
                }

            }
            _ => {
                html! {
                    <div>
                        { format!("{} members", num_members) }
                        <a class="show-removed-button", onclick=|_| Event::ToggleShowRemovedMembers,> {"Show Removed" } </a>
                    </div>
                }
            }
        };

        html! {
              <div>
                <div id="main-window",>
                      <div class="main-window-title",>
                          <span class="title",>{"Team"}</span>
                          <div class=("action-button", "main-window-action", "button"), onclick=|_| { js! {openAddTeam(); }; Event::Ignore },>
                              <a>{"ADD TEAM MEMBERS"}</a>
                          </div>
                      </div>
                      <div id="num-members", class="main-window-list-title",>
                          { num_members_html }
                      </div>

                      <div class="filters",>
                        { self.view_for_team_page_filters() }
                      </div>

                      <div class="list-container",>
                          <div class="list-header-user",>
                            {"USER"}
                          </div>
                          <div class="list-header-last-access",>
                                {"LAST ACCESS"}
                          </div>
                          <div class="list-header-logins-today",>
                                {"# REQUESTS"}
                          </div>
                          <div class="list-header-public-keys",>
                                {"KEYS"}
                          </div>
                          <div class="list-header-actions",>
                              {"ACTIONS"}
                          </div>
                          <div class="list-header-border",></div>

                            {
                                for filtered_members.iter().map(|m| view_for_team_list_item(m, me_clone.clone()) )
                            }

                      </div> // list-container
                        {
                            match search_query.clone().is_empty() {
                                false => {
                                  html! {<div id="search-results-footer", style="grid-column: span 12; text-align: center; opacity: 0.5; font-style: italic;",>
                                            { format!("Showing {} out-of {} results for search \"{}\"", filtered_members.len(), all_count, search_query) }
                                        </div>}
                                },
                                true => {html! {<div></div>} }
                            }
                        }
                    // side bar
                    <div id="sidebar-overlay", onclick=|_| { js! { closeSidebar(); }; Event::Ignore },></div>
                    { self.view_member_sidebar() }

              </div> // main window
            </div> // root
        }
    }
}
