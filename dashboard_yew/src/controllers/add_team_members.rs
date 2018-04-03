use yew::prelude::*;
use yew::format::{Json};
use yew::services::fetch::{Response, StatusCode};
use yew::services::console::ConsoleService;

use serde_json;

use context::*;
use models::*;

use sigchain_core::protocol::team::IndirectInvitationRestriction;

pub fn create_indirect_invitation(restriction:IndirectInvitationRestriction, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    model.fetching = true;
    model.krypton_request_state = KryptonAppRequestState::WaitingForResponse;

    let callback = context.send_back(move |response: Response<Json<Result<LinkResponse, ()>>>| {
        let (meta, Json(data)) = response.into_parts();

        if meta.status == StatusCode::FORBIDDEN {
            return Event::SessionExpired;
        }

        ConsoleService.log(&format!("{:?}", meta.status));
        match data {
            Ok(link_response) => {
                Event::Many(vec![
                    Event::RequestStateChanged(KryptonAppRequestState::Approved),
                    Event::DidCreateInviteURL(link_response.link),
                ])
            },
            Err(_) => {
                Event::Many(vec![
                    Event::ClearInviteURL,
                    Event::RequestStateChanged(KryptonAppRequestState::Rejected),
                ])
            },
        }
    });

    match serde_json::to_string(&restriction) {
        Ok(json_string) => {
            let request = model.build_post("/api/invite").body(json_string).unwrap();

            context.web.fetch(request, callback);
        }
        _ => {
            model.krypton_request_state = KryptonAppRequestState::Error("Request could not be created.".into());
        }
    }

    return true;
}
