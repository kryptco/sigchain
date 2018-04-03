use super::copy_helper::*;
use models::*;
use yew::prelude::*;
use context::*;
use sigchain_core::time_util::*;

pub fn view_for_is_admin(member:&TeamMember) -> Html<Context, Model> {
    if member.is_admin {
        html! {
           <span class=("role", "admin"),> { "ADMIN" }</span>
        }
    } else {
        html! {
           <span></span>
        }
    }
}

pub fn view_for_last_active(member:&TeamMember) -> Html<Context, Model> {
    match member.last_access {
        Some(ref access) => {
            html! {
                <div class="last-active",> {"Last used "} <span>{ access.unix_seconds.time_ago() }</span></div>
            }
        },
        _ => {
            html! { <div class="last-active",> </div> }
        }
    }
}

pub fn view_for_last_access(member:&TeamMember) -> Html<Context, Model> {
    match member.last_access {
        Some(ref access) => {
            html! {
                <div class="terminal",>
                     <span class="terminal-cmd",> {"$ "} </span>
                     { access.body.log_description() }
                </div>
            }
        },
        _ => {
            html! { <div> {"Hasn't used Krypton yet."} </div> }
        }
    }
}

pub fn view_for_role_change_button(member:&TeamMember) -> Html<Context, Model> {
    let the_member = member.clone();

    match member.is_admin {
        false => {
            html! {
                <button class="list-action-button", onclick=move |_| {  Event::Promote(the_member.clone()) },>{"PROMOTE"}</button>
            }
        },
        _ => {
            html! {
                <button class=("list-action-button", "remove"), onclick=move |_| {  Event::Demote(the_member.clone()) },>{"DEMOTE"}</button>
            }
        }
    }
}

pub fn view_for_remove_button(member:&TeamMember, is_me:bool) -> Html<Context, Model> {
    let the_member = member.clone();

    match member.is_removed || is_me {
        false => {
            html! {
                  <button class=("list-action-button", "remove"), onclick=move |_| {  Event::Remove(the_member.clone()) },>{"REMOVE"}</button>
            }
        },
        _ => {
            html! {
            <div></div>
            }
        }
    }
}

pub fn view_for_copy_buttons(member:&TeamMember) -> Html<Context, Model> {
    let ssh = member.identity.ssh_public_key.clone();
    let pgp = member.identity.pgp_public_key.clone();

    html! {
            <div class="list-item-public-keys",>
                <button class="public-key-button", onclick=move |_| {
                    copy_text_to_clipboard(&ssh.clone());
                    Event::Ignore
                },>{"SSH"}</button>
                <button class="public-key-button", onclick=move |_| {
                    copy_text_to_clipboard(&pgp.clone());
                    Event::Ignore
                },>{"PGP"}</button>
            </div>
    }

}


pub fn view_for_team_list_item(member:&TeamMember, me: Option<Identity>) -> Html<Context, Model> {
    let (user,domain) = member.get_user_and_mail_domain();
    let the_member:TeamMember = member.clone();
    let is_me = me.map(|m| m.public_key == member.identity.public_key).unwrap_or(false);

    if member.is_removed {
        html! {
            <div class=("list-item", "removed"),>
                <div class="list-item-user",>
                    <span class=("role","removed"),>{"REMOVED"}</span>
                    <span class="user-email", onclick=move |_| {
                                js! {openSidebar();};
                                Event::Many(vec![Event::ViewNextLogsPage(1),
                                                 Event::OpenTeamSidebar(the_member.clone())])
                    },>
                        { user.unwrap_or("--") }
                        <span class="mail-domain",> {"@"}{ domain.unwrap_or("--") }</span>
                    </span>
                    { view_for_last_active(member) }
                </div>

                <div class="list-item-last-access",>
                    { view_for_last_access(member) }
                </div>
                <div class="list-item-logins-today",>
                  { member.logins_today }
                </div>
                { view_for_copy_buttons(member) }
                <div class="list-item-actions",>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="list-item",>
                <div class="list-item-user",>
                    <span class="user-email", onclick=move |_| {
                                js! {openSidebar();};
                                Event::Many(vec![Event::ViewNextLogsPage(1),
                                                 Event::OpenTeamSidebar(the_member.clone())])
                    },>
                        { user.unwrap_or("--") } <span class="mail-domain",> {"@"}{ domain.unwrap_or("--") }</span>
                    </span>
                    { view_for_is_admin(member) }
                    { view_for_last_active(member) }
                </div>

                <div class="list-item-last-access",>
                    { view_for_last_access(member) }
                </div>
                <div class="list-item-logins-today",>
                  { member.logins_today }
                </div>
                { view_for_copy_buttons(member) }
                <div class="list-item-actions",>
                    { view_for_remove_button(member, is_me) }
                    { view_for_role_change_button(member) }
                </div>
            </div>
        }
    }
}
