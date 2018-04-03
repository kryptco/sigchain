use models::*;
use yew::prelude::*;
use context::*;

impl Model {
    pub fn view_for_hosts_page_filters(&self) -> Html<Context, Model> {
        html! {
              <div class="filter",>
                  <div class="icon-input",>
                      <img src="img/search.svg", class="icon",></img>
                      <input    id="host-page-search",
                                placeholder="Search by host name",
                                oninput=|_| {
                                    use stdweb::unstable::TryInto;
                                    let query:String = js! {
                                        return document.getElementById("host-page-search").value;
                                    }.try_into().unwrap_or("".into());
                                    println!("query: {}", query);
                                    Event::SearchChanged(query)
                                },>
                       </input>
                  </div>
              </div>
        }
    }
}
