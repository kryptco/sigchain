use yew::prelude::*;
use yew::format::{Nothing, Json};
use yew::services::fetch::{Response, StatusCode};
use std::time::Duration;

use context::*;
use models::*;

mod change_member_role;
use self::change_member_role::*;

mod remove_member;
use self::remove_member::*;

mod add_team_members;
use self::add_team_members::*;

mod handle_team_data;
use self::handle_team_data::*;

mod settings;
use self::settings::*;

mod log_chart;

pub use super::constants::*;

impl Component<Context> for Model {
    type Msg = Event;
    type Properties = ();

    fn create(_: &mut Env<Context, Self>) -> Self {
        Model::empty()
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Event::Initial => {
                self.fetching = true;

                let callback = context.send_back(move |response: Response<Json<Result<KryptonStatus, ()>>>| {
                    let (meta, Json(data)) = response.into_parts();

                    if meta.status == StatusCode::FORBIDDEN {
                        Event::SessionExpired
                    } else if let Ok(result_data) = data  {
                        println!("Got status: {:?}", result_data);

                        let event = match result_data {
                            KryptonStatus::Approved => {
                                Event::DoRequest(false)
                            }
                            KryptonStatus::NeedsApproval => {
                                Event::Many(vec![Event::RequestStateChanged(KryptonAppRequestState::WaitingForResponse), Event::CheckApprovalStatus, Event::DoRequest(false)])
                            }
                        };

                        event

                    } else {
                        Event::RequestStateChanged(KryptonAppRequestState::Error("Cannot load team data.".into()))
                    }
                });

                let request = self.build_get("/api/status").body(Nothing).unwrap();
                context.web.fetch(request, callback);

                return true;
            }
            Event::CheckApprovalStatus => {

                let callback = context.send_back(move |response: Response<Json<Result<KryptonStatus, ()>>>| {
                    let (meta, Json(data)) = response.into_parts();

                    if meta.status == StatusCode::FORBIDDEN {
                        Event::SessionExpired
                    } else if let Ok(result_data) = data  {
                        println!("Got status: {:?}", result_data);

                        let event = match result_data {
                            KryptonStatus::Approved => {
                                Event::RequestStateChanged(KryptonAppRequestState::Approved)
                            }
                            KryptonStatus::NeedsApproval => {
                                Event::EventAfter(5, vec![Event::CheckApprovalStatus])
                            }
                        };

                        event
                    } else {
                        Event::Ignore
                    }
                });

                let request = self.build_get("/api/status").body(Nothing).unwrap();
                context.web.fetch(request, callback);

                return true;
            }
            Event::EventAfter(delay, events) => {
                let event = events.clone().pop().unwrap_or(Event::Ignore);
                let callback = context.send_back(move |_| event.clone());
                context.timeout.spawn(Duration::from_secs(delay.clone()), callback);
                return true;
            }
            Event::DoRequest(is_recurring) => {
                // check window is in focus
                use stdweb::unstable::TryInto;
                let has_focus = js! {
                       if (document.hasFocus()) {
                          return true;
                       }
                       return false;
                }.try_into().unwrap_or(true);

                // if not in focus, delay request and check again
                if !has_focus {
                    self.update(Event::EventAfter(15, vec![Event::DoRequest(is_recurring)]), context);
                    return false;
                }

                self.fetching = true;
                let callback = context.send_back(move |response: Response<Json<Result<DashboardResponse, ()>>>| {
                    let (meta, Json(data)) = response.into_parts();

                    if meta.status == StatusCode::FORBIDDEN {
                        Event::SessionExpired
                    } else if let Ok(result_data) = data  {
                        Event::HandleResponse(result_data, is_recurring.clone())
                    } else {
                        Event::RequestStateChanged(KryptonAppRequestState::Error("Cannot load team data.".into()))
                    }
                });

                let request = self.build_get("/api/team").body(Nothing).unwrap();
                context.web.fetch(request, callback);
            }
            Event::HandleResponse(response, is_recurring) => {
                return handle_team_data(response, is_recurring, self, context);
            }
            Event::SessionExpired => {
                self.session_is_expired = true;
                return true;
            }
            Event::Ignore => {
                return false;
            }
            // side bars
            Event::OpenTeamSidebar(member) => {
                self.team_page.selected_member = Some(member);
                return true;
            }
            Event::CloseTeamSidebar => {
                self.team_page.selected_member = None;
                return true;
            }

            Event::OpenHostSidebar(host, tab) => {
                self.hosts_page.selected_host = Some(host);
                self.hosts_page.selected_sidebar_tab = tab;
                return true;
            }
            Event::CloseHostSidebar => {
                self.hosts_page.selected_host = None;
                return true;
            }

            // member handlers
            Event::Remove(member) => {
                return handle_remove_member(member, self, context);
            }
            Event::Demote(member) | Event::Promote(member) => {
                return change_member_role(member, self, context);
            }

            // Invite handlers
            Event::CreateInvite(restriction) => {
                js! {  createLink(); }
                self.team_page.handles_request_state = true;
                return create_indirect_invitation(restriction, self, context);
            }
            Event::DidCreateInviteURL(url) => {
                js! {  showLink(); }
                self.team_page.invite_link = Some(url);
                return true;
            }
            Event::ClearInviteURL => {
                self.team_page.invite_link = None;
                self.team_page.handles_request_state = false;
                return true;
            }

            //settings helpers
            Event::SetTeamName(name) => {
                return change_team_name(name, self, context);
            }
            Event::EditApprovalWindow(seconds) => {
                return edit_approval_window(seconds, self, context);
            }
            Event::EnableAuditLogging(should_enable) => {
                return enable_logging_endpoint(should_enable, self, context);
            }
            Event::ToggleIsEditingSetting(is_editing) => {
                self.settings_page.is_editing = is_editing;
                return true;
            }
            Event::DrawChart => {
                let chart_value_strings:Vec<String> = self.logs_page.log_chart_values.clone().into_iter().map(|v| format!("{}", v) ).collect();
                let chart_value_js = chart_value_strings.join(",").clone();

                js! {
                    if (chart) {
                        var theChart = chart;
                        theChart.data.series[0].data = [];

                        var data = @{chart_value_js};
                        var new_vals = data.split(",");
                        for (i = 0; i < new_vals.length; i++) {
                            var val = parseInt(new_vals[i],10);
                            theChart.data.series[0].data.push(val);
                        }

                        theChart.update();
                    } else {
                            console.log("chart doesn't exist yet");
                    }
                };
                return true
            }
            // Page
            Event::SelectPage(page) => {
                // reset the log page limit
                self.logs_page_limit = 1;

                self.selected_page = page.clone();
                if Page::AuditLogs == page {
                    let callback = context.send_back(move |_| Event::DrawChart);
                    context.timeout.spawn(Duration::from_millis(100), callback);
                }
                return true;
            }

            // Logs pagination
            Event::ViewNextLogsPage(next_page_limit) => {
                self.logs_page_limit = next_page_limit;
                return true;
            }

            // Searching
            Event::SearchChanged(query) => {
                match self.selected_page {
                    Page::Team => {
                        self.team_page.search_query = query;
                    }
                    Page::Hosts => {
                        self.hosts_page.search_query = query;
                    }
                    Page::AuditLogs => {
                        self.logs_page.search_query = query;
                    }
                    _=> {}
                };
                return true;
            },

            Event::ToggleShowRemovedMembers => {
                self.team_page.show_removed = !self.team_page.show_removed;
            },

            // Loader/Krypton state
            Event::RequestStateChanged(new_state) => {
                match new_state {
                    KryptonAppRequestState::Approved => {
                        context.console.log("hiding approval state in several seconds");

                        let callback = context.send_back(|_| Event::RequestStateChanged(KryptonAppRequestState::None));
                        context.timeout.spawn(Duration::from_secs(4), callback);

                        self.krypton_request_state = new_state;
                        return true;
                    },
                    KryptonAppRequestState::Rejected => {
                        js! { addCloseIcon(); allowAddTeamClose(); closeAddTeam(); };
                        context.console.log("hiding approval state in several seconds");

                        let callback = context.send_back(|_| Event::RequestStateChanged(KryptonAppRequestState::None));
                        context.timeout.spawn(Duration::from_secs(4), callback);

                        self.krypton_request_state = new_state;
                        return true;
                    },
                    _ => {
                        self.krypton_request_state = new_state;
                        return true;
                    }
                };
            }

            // helpers
            Event::Many(events) => {
                for event in events {
                    self.update(event, context);
                }
            }
        }
        true
    }
}
