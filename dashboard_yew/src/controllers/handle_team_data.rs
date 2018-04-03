use yew::prelude::*;
use std::time::Duration;

use sigchain_core::protocol::logs::{LogBody, SSHSignatureResult};
use std::collections::HashMap;

use context::*;
use models::*;
use super::log_chart::*;

pub fn handle_team_data(response:DashboardResponse, is_recurring: bool, model: &mut Model, context: &mut Env<Context, Model>) -> ShouldRender {
    context.console.log("got handle response");

    model.show_cover_splash = false;
    model.fetching = false;

    // Handle the response
    model.me = Some(response.me.clone());
    model.team_name = response.team_name.clone();
    model.temporary_approval_seconds = response.temporary_approval_seconds;
    model.audit_logging_enabled = response.audit_logging_enabled;
    model.billing_data = response.billing_data.clone();
    model.is_data_fresh = response.data_is_fresh;
    model.all_new_logs_loaded = response.all_new_logs_loaded;

    // update the team list
    model.members = response.team_members.clone();

    // update the selected member if there is one
    model.team_page.selected_member.clone().map(|sm| {
        model.team_page.selected_member = model.members.iter().filter(|m| {
            m.identity.public_key == sm.identity.public_key
        }).next().cloned();
    });

    js! { hideSplash(); }


    // if we were waiting for a krypton app response, update the state
    match (model.krypton_request_state.clone(), is_recurring.clone()) {
        (KryptonAppRequestState::LoadingResult, false) => {
            model.update(Event::RequestStateChanged(KryptonAppRequestState::Approved), context);
        }
        (KryptonAppRequestState::WaitingForResponse, false) => {
            model.update(Event::RequestStateChanged(KryptonAppRequestState::Approved), context);
        }
        _ => {}
    };

    //TODO: separate logs request
    let mut all_logs:Vec<LogByUser> = Vec::new();
    for member in &response.team_members {
        for log in &member.last_24_hours_accesses {
            all_logs.push(LogByUser { log: log.clone(), member_email: member.identity.email.clone() });
        }
    }
    all_logs.sort_by(|a, b| {
        b.log.unix_seconds.cmp(&a.log.unix_seconds)
    });

    //TODO: separate hosts request

    let mut host_log_map:HashMap<String, Vec<LogByUser>> = HashMap::new();
    let mut host_people_map:HashMap<String, Vec<&TeamMember>> = HashMap::new();

    for member in &response.team_members {
        for log in &member.last_24_hours_accesses {
            match log.body {
                LogBody::Ssh(ref signature) => {
                    match signature.result {
                        SSHSignatureResult::Signature(_) => {}
                        _=>{ continue; }
                    };
                    let member_email = &member.identity.email;
                    match signature.host_authorization {
                        Some(ref host_auth) => {
                            host_log_map.entry(host_auth.host.clone()).or_insert(vec![]).push(
                                LogByUser { log: log.clone(), member_email: member_email.clone() }
                            );

                            host_people_map.entry(host_auth.host.clone()).or_insert(vec![]).push(member);
                        }
                        _ => { continue; }
                    };
                }
                _ => { continue; }
            };
        }
    }


    let mut hosts:Vec<Host> = Vec::new();

    for (host_name, mut logs) in host_log_map {
        let members:Vec<&TeamMember> = match host_people_map.get(&host_name) {
            None => { continue; }
            Some(member_list) => {
                member_list.clone()
            }
        };

        let mut people:Vec<TeamMemberForHost> = members.into_iter().map(|m| {
            let mut last_access:Vec<i64> = logs.iter().filter(|l| l.member_email == m.identity.email ).map(|l| l.log.unix_seconds as i64).collect();
            last_access.sort_by(|a,b| a.cmp(&b) );
            TeamMemberForHost { member: m.identity.clone(),
                                last_access_unix_seconds: *last_access.first().unwrap_or(&0),
                                num_accesses: last_access.len() as u64 }
        }).collect();

        people.dedup_by(|a,b| a.member.public_key == b.member.public_key);

        let mut last_access:Vec<i64> = logs.iter().map(|l| l.log.unix_seconds as i64).collect();
        last_access.sort_by(|a,b| a.cmp(&b));

        logs.sort_by(|a,b| b.log.unix_seconds.cmp(&a.log.unix_seconds));
        hosts.push(Host {   domain: host_name.clone(),
                            people: people,
                            logs: logs,
                            last_access_unix_seconds: *last_access.first().unwrap_or(&0)
        });
    }

    hosts.sort_by(|a,b| b.last_access_unix_seconds.cmp(&a.last_access_unix_seconds));

    // update the selected host if needed
    model.hosts_page.selected_host.clone().map(|sh| {
        model.hosts_page.selected_host = hosts.iter().filter(|h| {
            h.domain == sh.domain
        }).next().cloned();
    });

    model.hosts_page.hosts = hosts;

    // schedule next teams_request
    if response.all_new_logs_loaded {
        let callback = context.send_back(move |_| Event::DoRequest(true));
        let handle = context.timeout.spawn(Duration::from_secs(15), callback);
        model.pending_job = Some(Box::new(handle));
        context.console.log("scheduled next teams request in 15s");
    } else {
        let callback = context.send_back(move |_| Event::DoRequest(true));
        let handle = context.timeout.spawn(Duration::from_millis(100), callback);
        model.pending_job = Some(Box::new(handle));
        context.console.log("more logs to load, scheduling next teams request now");
    }


    // update chart
    model.logs_page.log_chart_values = create_log_chart_values(all_logs.clone());
    if Page::AuditLogs == model.selected_page {
        let callback = context.send_back(move |_| Event::DrawChart);
        context.timeout.spawn(Duration::from_millis(100), callback);
    }

    model.logs_page.logs = all_logs;

    return  true;
}