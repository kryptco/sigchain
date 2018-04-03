use context::*;
use models::*;
use yew::prelude::*;
use time::Duration;
use std::ops::Sub;

fn view_for_admin_setting_item(admin:TeamMember) -> Html<Context, Model> {
    let (user,domain) = admin.get_user_and_mail_domain();
    let admin_clone = admin.clone();
    html! {
        <div class="box",>
            <div class="title",>
                {user.unwrap_or("--")} <span class="mail-domain",>{"@"} {domain.unwrap_or("--")} </span>
            </div>
            <div class=("option", "right"),>
                <button class=("list-action-button", "remove"),
                        onclick=move |_| Event::Demote(admin_clone.clone()),>
                    {"Remove Access"}
                </button>
            </div>
        </div>
    }
}

fn view_for_approval_window_item(unix_seconds:Option<i64>) -> Html<Context, Model> {
    let (hours, minutes) = match unix_seconds {
        Some(seconds) => {
            let duration = Duration::seconds(seconds);
            let hours = duration.num_hours();
            let minutes = duration.sub(Duration::hours(hours)).num_minutes();
            (hours, minutes)
        },
        None => {
            (0, 0)
        },
    };

    html! {
        <div class="title",>
            <span id="status-label", style=format!("display: {};", match unix_seconds {
                Some(0) => "inline",
                Some(_) => "none",
                None => "inline",
            }),>{match unix_seconds {
                Some(0) => "DISABLED",
                Some(_) => "",
                None => "NOT SET",
            }}</span>
            <span id="hr-text", style=format!("color: #5BC894; font-size: 1.2em; font-weight: 600; display: {};", match unix_seconds {
                Some(_) => if hours == 0 {
                    "none"
                } else {
                    "inline"
                },
                None => "none",
            }),>{hours}</span>
            <input  type="number",
                    class=("title", "disabled"),
                    value={hours},
                    id="window-hr-input",
                    style="width: 3em; text-align: right; color: #5BC894; font-size: 1.2em; font-weight: 600;",
                    min="0",
                    max="23",>
            </input>
            <span id="hour-label", style=format!("padding-right: 4px; display: {};", match hours {
                0 => "none",
                _ => "inline",
            }),> {" hours"} </span>
            <span id="min-text", style=format!("color: #5BC894; font-size: 1.2em; font-weight: 600; display: {};", match unix_seconds {
                Some(_) => if minutes == 0 {
                    "none"
                } else {
                    "inline"
                },
                None => "none",
            }),>{minutes}</span>
            <input  type="number",
                    class=("title", "disabled"),
                    value= {minutes},
                    id="window-min-input",
                    style="width: 3em; text-align: right; color: #5BC894; font-size: 1.2em; font-weight: 600;",
                    min="0",
                    max="59",>
            </input>
            <span id="min-label", style=format!("display: {};", match minutes {
                0 => "none",
                _ => "inline",
            }),> {" min"} </span>
        </div> // title
    }
}

fn view_for_logs_enable_setting(is_enabled: bool) -> Html<Context, Model> {
    return match is_enabled {
        true => {
            html! {
                <div class="box",>
                    <div class="title",>
                        {"Enabled"}
                    </div>
                    <div class=("option", "right"),>
                        <button class=("list-action-button", "remove"),
                        		onclick=|_| {
                        		js!{ alert("Disabling audit logging is only currently supported in the Krypton phone app.");};
                        		Event::Ignore },>
                        {"Disable"}</button>
                    </div>
                </div>
                }
        }
        false => {
            html! {
                <div class="box",>
                    <div class="title",>
                        {"Disabled"}
                    </div>
                    <div class=("option", "right"),>
                        <button class=("list-action-button"), onclick=|_| {Event::EnableAuditLogging(true)},>
						{"Enable"}</button>
                    </div>
                </div>
            }
        }
    };
}

impl Model {

    fn view_for_billing_setting(&self) -> Html<Context, Model> {
        let billing_url = self.billing_data.url.clone();
        let tier_name = self.billing_data.billing_info.current_tier.name.clone();

        let usage_members = self.billing_data.billing_info.usage.members;
        let usage_hosts = self.billing_data.billing_info.usage.hosts;
        let usage_logs = self.billing_data.billing_info.usage.logs_last_30_days;

        let tier_members = self.billing_data.billing_info.current_tier.limit.members;
        let tier_hosts = self.billing_data.billing_info.current_tier.limit.hosts;
        let tier_logs = self.billing_data.billing_info.current_tier.limit.logs_last_30_days;

        // check if tier paid
        let tier_class = match self.billing_data.is_paid() {
            true => "paid",
            false => ""
        };

        let action_button_text = match self.billing_data.is_paid() {
            true => "Manage Billing",
            false => "Upgrade"
        };

        // emphasize usage that's close to the limit
        let members_emph_class = match usage_members > tier_members/2 {
            true => "tier-close",
            false => ""
        };
        let hosts_emph_class = match usage_hosts > tier_hosts/2 {
            true => "tier-close",
            false => ""
        };
        let logs_emph_class = match usage_logs > tier_logs/2 {
            true => "tier-close",
            false => ""
        };

        let is_paid_clone = self.billing_data.is_paid().clone();

        html! {
                 <div class=("setting", "box"),>
                    <div class=("setting", "title"),>
                        <span class="billing-header",>{"Billing & Plan"}</span>
                        <span class=("tier", tier_class),>
                            { tier_name }
                        </span>
                        <div class=("setting", "meta"),>
                            <div class="tier-usage",>
                                <div class="tier-usage-title",>{"Team Members"}</div>
                                <div class="tier-usage-value",><span class=("usage", members_emph_class),>{ usage_members }</span><span class="divisor",>{"/"}</span><span class="limit",>{ tier_members }</span></div>
                            </div>
                            <div class="tier-usage",>
                                <div class="tier-usage-title",>{"Shared Known Hosts"}</div>
                                <div class="tier-usage-value",><span class=("usage", hosts_emph_class),>{ usage_hosts }</span><span class="divisor",>{"/"}</span><span class="limit",>{ tier_hosts }</span></div>
                            </div>
                            <div class="tier-usage",>
                                <div class="tier-usage-title",>{"Monthly Audit Logs"}</div>
                                <div class="tier-usage-value",><span class=("usage", logs_emph_class),>{ usage_logs }</span><span class="divisor",>{"/"}</span><span class="limit",>{ tier_logs/1000 }{"k"}</span></div>
                            </div>
                        </div>
                    </div>
                    <div class=("upgrade"),>
                        <button id="upgrade-button", class=(tier_class), onclick=move |_| {
                                if is_paid_clone {
                                    js! { alert("Billing management in-dashboard is coming soon! Please email support@krypt.co for billing questions."); }
                                    return Event::Ignore;
                                }

                                js! { window.open(@{billing_url.clone()}); };
                                Event::Ignore
                            },>
                            { action_button_text }
                        </button>
                    </div>
                </div> // box
            }

    }

    pub fn view_for_settings_page(&self) -> Html<Context, Model> {
        let me_email = self.me.clone().map(|m| m.email).unwrap_or("".into());

        // other admins
        let admins:Vec<TeamMember> = self.members.clone().into_iter().filter(|m| m.is_admin && (m.identity.email != me_email)).collect();

        let mut tokens = me_email.split("@");
        let (me_user, me_domain) = (tokens.next(), tokens.next());

        let approval_window = self.temporary_approval_seconds.clone();

        let is_editing = self.settings_page.is_editing.clone();
        let existing_name = self.team_name.clone();
        html! {
              <div id="main-window",>
                  <div class="main-window-title",>
                      <span class="title",>{"Settings"}</span>
                  </div> //main-window-title
                  <div class="settings",>
					<div class=("setting", "box"),>
						<div class=("setting", "title"),>
							{"Team name"}
						</div>
						<div class=("setting", "card"),>
							<div class="box",>
								<input  type="text",
								        class=("title", "disabled"),
								        value={ self.team_name.clone() },
								        id="team-input",
								        readonly="true",>
								</input>
								<div class=("option", "right"),>
									<button class=("button", "list-action-button"),
									        id="team-edit",
									        onclick=move |_| {
									            js!{editTeam();};

									            return match is_editing.clone() {
									                false => {  Event::ToggleIsEditingSetting(true) }
									                true => {
									                            use stdweb::unstable::TryInto;
                                                                let new_name:String = js! {
                                                                    return document.getElementById("team-input").value;
                                                                }.try_into().unwrap_or(existing_name.clone());
                                                                Event::Many(vec![Event::ToggleIsEditingSetting(false), Event::SetTeamName(new_name)])
									                }
									            };
									        },>
									        { "Edit" }
									</button>
								</div>
							</div> // box
						</div> // card
					</div> // setting box
					<div class=("setting", "box"),>
						<div class=("setting", "title"),>
							{"Auto-Approval Window"}
						    <div class=("setting", "meta"),>
								{"The auto-approval window determines the length of time that requests for SSH authentication and Git code signing will be approved automatically without user interaction. Shorter times are more secure. "}
							</div>
						</div>
						<div class=("setting", "card"),>
							<div class="box",>
								{ view_for_approval_window_item(approval_window.clone()) }
                                <div class=("option", "right"),>
									<button class="list-action-button",
									        onclick=move |_|
									        {
									            js!{ editWindow(); };

									            return match is_editing.clone() {
									                false => {  Event::ToggleIsEditingSetting(true) }
									                true => {
									                            use stdweb::unstable::TryInto;
                                                                let new_seconds:i64 = js! {
                                                                    var hour_seconds = parseInt(document.getElementById("window-hr-input").value, 10)*3600;
                                                                    var min_seconds = parseInt(document.getElementById("window-min-input").value, 10)*60;
                                                                    return hour_seconds + min_seconds;
                                                                }.try_into().unwrap_or(-1);
                                                                if new_seconds < 0 {
                                                                    Event::Many(vec![Event::ToggleIsEditingSetting(false), Event::EditApprovalWindow(approval_window.clone())])
                                                                } else {
                                                                    Event::Many(vec![Event::ToggleIsEditingSetting(false), Event::EditApprovalWindow(Some(new_seconds.abs()))])
                                                                }

									                }
									            };

									        },
									        id="window-edit",>

									        {"Edit"}
									</button>
                                    <button class="list-action-button", style=format!("margin-left: 3px; display: {};", match approval_window {
                                            Some(_) => "inline",
                                            None => "none",
                                        }),
									        onclick=move |_|
									        {
									            js!{ unsetWindow(); };
                                                return Event::Many(vec![Event::ToggleIsEditingSetting(false), Event::EditApprovalWindow(None)])
									        },>

									        {"Unset"}
									</button>
								</div>

							</div> // box
						</div> // card
					</div> // setting box

					<div class=("setting", "box"),>
						<div class=("setting", "title"),>
							{"Audit logging"}
						<div class=("setting", "meta"),>
								{"Enable to see the access logs (SSH logins and Git code signatures) of your team members. \
								Audit logs are encypted ONLY to team admins. Neither member's nor krypt.co have access your teams' audit logs."}
							</div>
						</div>
						<div class=("setting", "card"),>
						    { view_for_logs_enable_setting(self.audit_logging_enabled.clone()) }
						</div>
					</div>
					{ self.view_for_billing_setting() }
					<div class=("setting", "box"), style="border-bottom: none;",>
						<div class=("setting", "title"),>
							{"Team admins"}
						<div class=("setting", "meta"),>
								{"Team admins can set team policies, generate invite links, make other users admins, pin known hosts and read logs (if enabled)."}
							</div>
						</div>
						<div class=("setting", "card"),>
							<div class="box",>
								<div class="title",>
									{me_user.unwrap_or("--")}<span class="mail-domain",>{"@"}{me_domain.unwrap_or("--")}</span>
								</div>
								<div class=("option", "right"),>
									{"You"}
								</div>
							</div>
							{
							    for admins.into_iter().map(|a| view_for_admin_setting_item(a))
							}
						</div>
					</div>


                </div> // settings
              </div> // main window
        }
    }
}
