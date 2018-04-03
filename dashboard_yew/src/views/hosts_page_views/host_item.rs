use models::*;
use yew::prelude::*;
use context::*;
use sigchain_core::time_util::*;

pub fn view_for_host_item(host:&Host) -> Html<Context, Model> {
    let the_host = host.clone();
    let domain = host.domain.clone();
    let people_count = host.people.len();
    let access_count = host.logs.len();

    let the_host_people = the_host.clone();
    let the_host_logs = the_host.clone();

    html! {
            <div class="list-item",>
                <div class="list-item-host",>
                    <span class="host-name", onclick=move |_| {
                        js! {openSidebar();};
                        Event::Many(vec![
                            Event::OpenHostSidebar(the_host.clone(), HostSidebarTab::People),
                            Event::ViewNextLogsPage(1),
                        ])
                     },>
                      { domain.clone() }
                    </span>
                </div>

                <div class="list-item-people",>
                    <span class="data",>{ people_count } </span>
                    <div class="last-active",>
                        <a onclick=move |_| { js!{openSidebar();}; Event::OpenHostSidebar(the_host_people.clone(), HostSidebarTab::People) },>
                            {"View people"}
                        </a>
                    </div>
                </div>
                <div class="list-item-accesses",>
                    <span class="data",> { access_count }</span> <span class=("meta", "last-active"),></span>
                    <div class="last-active",>
                        <a onclick=move |_| {
                            js!{openSidebar();};
                            Event::Many(vec![
                                Event::OpenHostSidebar(
                                    the_host_logs.clone(),
                                    HostSidebarTab::Logs,
                                ),
                                Event::ViewNextLogsPage(1),
                            ])
                         },>
                            {"View logs"}
                        </a>
                    </div>
                </div>
                <div class="list-item-accesses",>
                    <span class="list-item-host-last-access",> { host.last_access_unix_seconds.time_ago() }</span>
                </div>
            </div>
        }
}
