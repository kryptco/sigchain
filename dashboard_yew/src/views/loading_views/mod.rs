use yew::prelude::*;
use context::*;
use models::*;

impl Model {
    pub fn view_for_bar_loader(&self) -> Html<Context, Model> {
        if self.fetching && !self.show_cover_splash {
            html! {
                <div id="loading-bar",>
                    <div id="loaded",></div>
                </div>
            }
        } else {
            html! { <div></div>}
        }
    }

    pub fn view_for_request_loader(&self) -> Html<Context, Model> {

        let request_state:KryptonAppRequestState;

        if self.team_page.handles_request_state {
            request_state = KryptonAppRequestState::None;
        } else {
            request_state = self.krypton_request_state.clone();
        }
        println!("Request state: {:?}", request_state);

        match request_state {
            KryptonAppRequestState::Loading => {
                html! {
                     <div id="loader",>
                      <div id="loader-bg", class="show",></div>
                        <div id="request-loader", class=("inner-loader", "show"),>
                            <div class="loader",></div>
                            <div class="loading-text",>
                                { "Requesting Approval from Krypton" }
                         </div>
                        </div>
                      </div>
                }
            },
            KryptonAppRequestState::LoadingResult => {
                html! {
                     <div id="loader",>
                      <div id="loader-bg", class="show",></div>
                        <div id="request-loader", class=("inner-loader", "show"),>
                            <div class="loader",></div>
                            <div class="loading-text",>
                                { "Loading data" }
                         </div>
                        </div>
                      </div>
                }
            },
            KryptonAppRequestState::Approved => {
                html! {
                        <div id="loader",>
                            <div id="loader-bg", class="show",></div>
                            <div id="success-loader", class=("inner-loader", "show"),>
                                <img src="img/success.svg",></img>
                                <div class="loading-text",>
                                    { "Success" }
                                    <div class="meta",>{ "Request Allowed" }</div>
                                </div>
                            </div>
                        </div>
                }
            }
            KryptonAppRequestState::WaitingForResponse => {
                html! {
                     <div id="loader",>
                        <div id="loader-bg", class="show",></div>
                          <div id="approve-loader", class=("inner-loader", "show"),>
                            <img src="img/phone_warning.svg",></img>
                             <div class="loading-text",>
                                 { "Phone Approval Required" }
                             </div>
                             <div class="meta",> { "Please respond using the Krypton app" }</div>
                          </div>
                     </div>
                }
            },
            KryptonAppRequestState::Rejected => {
                html! {
                     <div id="loader",>
                          <div id="loader-bg", class="show",></div>
                          <div id="fail-loader", class=("inner-loader", "show"),>
                              <img src="img/danger.svg",></img>
                              <div class="loading-text",>
                                  { "Approval Denied" }
                              </div>
                          </div>
                     </div>
                }
            },
            KryptonAppRequestState::Error(ref error) => {
                html! {
                     <div id="loader",>
                          <div id="loader-bg", class="show",>
                          </div>
                          <div id="error-loader", class=("inner-loader", "show"),>
                              <img src="img/danger.svg",></img>
                              <div class="loading-text",>
                                  { "Something went wrong: " } { error }
                                  {" Please refresh the page."}
                              </div>
                          </div>
                     </div>
                }
            },
            KryptonAppRequestState::None => {
                html! { <div> </div>}
            }
        }
    }
}
