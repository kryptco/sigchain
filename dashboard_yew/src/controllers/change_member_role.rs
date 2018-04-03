use yew::prelude::*;
use yew::format::{Json};
use yew::services::fetch::{Response, StatusCode};

use serde_json;

use context::*;
use models::*;

pub fn change_member_role(member:TeamMember, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    if member.is_admin {
        return change_member_role_to(member, "demote", model, context);
    } else {
        return change_member_role_to(member, "promote", model, context);
    }
}

fn change_member_role_to(member:TeamMember, endpoint:&str, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    model.cancel_task_if_present();
    model.fetching = true;

    model.update(Event::RequestStateChanged(KryptonAppRequestState::WaitingForResponse), context);

    let callback = context.send_back(move |response: Response<Json<Result<(), ()>>>| {
        let (meta, _) = response.into_parts();

        if meta.status == StatusCode::FORBIDDEN {
            return Event::SessionExpired;
        }

        match meta.status.is_success() {
            true => { Event::Many(vec!(Event::RequestStateChanged(KryptonAppRequestState::LoadingResult),
                                       Event::DoRequest(false)))
            }
            // todo handle actual errors versus just rejects
            false => {Event::RequestStateChanged(KryptonAppRequestState::Rejected)}
        }
    });

    let public_key_request = PublicKeyRequest { public_key: member.identity.public_key };
    match serde_json::to_string(&public_key_request) {
        Ok(json_string) => {
            let url = format!("/api/{}", endpoint);
            let request = model.build_post(url.as_str()).body(json_string).unwrap();
            context.web.fetch(request, callback);
        }
        _ => {
            model.krypton_request_state = KryptonAppRequestState::Error("Request could not be created.".into());
        }
    }
    return true;
}
