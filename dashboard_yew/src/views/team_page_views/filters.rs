use models::*;
use yew::prelude::*;
use context::*;

impl Model {
    pub fn view_for_team_page_filters(&self) -> Html<Context, Model> {
        html! {
              <div class="filter",>
                  <div class="icon-input",>
                      <img src="img/search.svg", class="icon",></img>
                      <input    id="team-page-search",
                                placeholder="Search by email",
                                oninput=|_| {
                                    use stdweb::unstable::TryInto;
                                    let query:String = js! {
                                        return document.getElementById("team-page-search").value;
                                    }.try_into().unwrap_or("".into());
                                    println!("query: {}", query);
                                    Event::SearchChanged(query)
                                },></input>
                  </div>
              </div>
        }
    }
}
