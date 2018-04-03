use models::*;
use yew::prelude::*;
use context::*;

impl Model {
    pub fn view_for_logs_page_filters(&self) -> Html<Context, Model> {
        html! {
              <div class=("filters", "right"),>
                  <div class="filter",>
                      <div class="icon-input",>
                          <img src="img/search.svg", class="icon",></img>
                          <input    id="logs-page-search",
                                    placeholder="Search by log details",
                                    oninput=|_| {
                                        use stdweb::unstable::TryInto;
                                        let query:String = js! {
                                            return document.getElementById("logs-page-search").value;
                                        }.try_into().unwrap_or("".into());
                                        println!("query: {}", query);
                                        Event::SearchChanged(query)
                                    },>
                           </input>
                      </div>
                  </div>
               </div>
        }
    }
}
