use super::copy_helper::*;
use models::*;
use yew::prelude::*;
use context::*;
use sigchain_core::protocol::team::IndirectInvitationRestriction;

impl Model {
    pub fn view_for_add_members(&self) -> Html<Context, Model> {
        let email_domain:String = match self.me {
            Some(ref me) => {
                me.email.split("@").nth(1).unwrap_or("domain").into()
            }
            _ => {
                "domain".into()
            }
        };

        let email_domain_cloned = email_domain.clone();
        let email_domain_label_cloned = email_domain_cloned.clone();

        let link_raw = self.team_page.invite_link.clone().unwrap_or("<error>".into());

        let link_text = format!("You're invited to join {} on Krypton!\n\
        Step 1. Install: https://get.krypt.co\n\
        Step 2. Join: Tap the link below on your phone or copy this message (including the link) into Krypton.\n\
        {}", self.team_name, link_raw);

        html! {
            <div>
                 <div id="team-bg", onclick=|_| { js! { closeAddTeam(); }; Event::Ignore },></div>
                    <div id="team-container", class="card",>
                        <div class="content",>
                            <div class="title", style="text-align: center;",> { "Add Team Members" } </div>
                            <div class="meta", style="text-align: center;",> { "How would you like to invite team members?" } </div>
                            <img src="img/x.svg", id="close-icon", class=("close", "icon"), onclick=|_| { js! { closeAddTeam(); }; Event::Ignore },></img>
                            <div id="options",>
                               <div class="option", onclick=move |_| {
                                                                        js! { openTeamLink(); };
                                                                        Event::CreateInvite(IndirectInvitationRestriction::Domain(email_domain_cloned.clone()))
                                                                      },>
                                    <div class="link",>
                                        <div class="icon",><img src="img/team_link.svg",></img></div>
                                        <a>{ "Team Link" }</a>
                                    </div>
                                    <div class="meta",>
                                        { "Anyone with an @"} { email_domain_label_cloned } {" email address"}
                                    </div>
                                </div>
                                <div class="option", onclick=|_| { js! { openIndLink(); }; Event::Ignore },>
                                    <div class="link",>
                                        <div class="icon",>
                                            <img src="img/email_link.svg",></img>
                                        </div>
                                        <a>{ "Individual Link" }</a>
                                    </div>
                                    <div class="meta",> { "Only specific individuals (by email address)" }</div>
                                </div>
                            </div>
                            <div id="ind-title", class="link-title",>
                                { "Individual link" }
                                <p> { "Enter the email addresses new team members will sign up with." }</p>
                            </div>

                            <div id="team-title", class="link-title",>
                                { "Team Link" }
                                <p> {"Anyone with an"} <span id="team-email",>{"@"} { email_domain }</span> {" address can sign up with this link."}</p>
                            </div>
                        </div>
                        <div id="option-details",>
                            <div class="content",>
                                <div id="link-load",>
                                    <div class="ball-holder",>
                                        <div class=("ball", "one"),></div>
                                        <div class=("ball", "two"),></div>
                                        <div class=("ball", "three"),></div>
                                    </div>
                                    <span id="link-load-text",> {"Requesting approval from Krypton"} </span>
                                </div>
                                <div id="enter-ind",>
                                    <form onsubmit="addInd(); return false;",>
                                        <input class="link", type="email", id="ind-email", placeholder="alice@acme.co",></input>
                                        <input type="submit", class=("action-button", "subtle"), onclick=|_| { js! { addInd(); }; Event::Ignore }, value="Add",></input>
                                    </form>
                                    <div class="list-title",> { "Member emails" }</div>
                                    <ul id="emails",></ul>
                                </div>
                                <div id="add-link",>
                                    <input class="link", id="link-url", value={ link_raw.clone() },></input>
                                    <button class=("action-button", "subtle"), onclick=move |_| {
                                        copy_text_to_clipboard(&link_text.clone());
                                        Event::Ignore
                                    },>{ "Copy" }</button>
                                    <a id="mail-link", class="action-button",> { "Email Invite" }</a>
                                </div>
                            </div>
                        </div>
                        <div id="ind-create",>
                            <button class=("action-button", "right"),
                                    onclick=|_| {
                                        use stdweb::unstable::TryInto;;
                                        js! { createIndLink();};
                                        let emails:Vec<String> = js! { return getEmails(); }.try_into().unwrap_or(vec!());
                                        Event::CreateInvite(IndirectInvitationRestriction::Emails(emails.clone()))
                                    }
                            ,> {"Create"}</button>
                        </div>
                    </div>
            </div>
        }
    }
}
