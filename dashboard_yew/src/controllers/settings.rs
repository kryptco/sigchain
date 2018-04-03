use yew::prelude::*;
use yew::format::{Json};
use yew::services::fetch::{Response, StatusCode};
use yew::services::console::ConsoleService;

use sigchain_core::protocol::team::{TeamInfo, Policy, LoggingEndpoint, E};

use serde_json;

use context::*;
use models::*;

use serde::ser::Serialize;

pub fn change_team_name(name:String, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    if name == model.team_name {
        return false;
    }

    do_post_request(&TeamInfo { name }, "team_info".into(), model, context);
    return true;
}

pub fn edit_approval_window(seconds:Option<i64>, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    if seconds == model.temporary_approval_seconds {
        return false;
    }

    do_post_request(&Policy { temporary_approval_seconds: seconds }, "policy".into(), model, context);
    return true;
}

pub fn enable_logging_endpoint(enable: bool, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    if model.audit_logging_enabled == enable {
        return false;
    }

    match enable {
        true => {
            do_post_request(&LoggingEndpoint::CommandEncrypted(E{}), "enable_logging".into(), model, context);
        }
        false => {
            do_post_request(&LoggingEndpoint::CommandEncrypted(E{}), "disable_logging".into(), model, context);
        }
    }

    return true;
}


fn do_post_request<T>(payload: &T, endpoint:String, model: &mut Model, context: &mut Env<Context, Model>)
where T: Serialize,
{
    model.cancel_task_if_present();
    model.fetching = true;

    model.update(Event::RequestStateChanged(KryptonAppRequestState::WaitingForResponse), context);

    let callback = context.send_back(move |response: Response<Json<Result<(), ()>>>| {
        let (meta, _) = response.into_parts();

        ConsoleService.log(&format!("{:?}", meta.status));

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

    match serde_json::to_string(payload) {
        Ok(json_string) => {
            let request = model.build_post(format!("/api/{}", endpoint).as_str()).body(json_string).unwrap();
            context.web.fetch(request, callback);
        }
        _ => {
            model.krypton_request_state = KryptonAppRequestState::Error("Request could not be created.".into());
        }
    }

}