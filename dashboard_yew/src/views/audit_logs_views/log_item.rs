use models::*;
use yew::prelude::*;
use context::*;
use sigchain_core::time_util::*;
use sigchain_core::protocol::logs::*;
use sigchain_core::git_hash::*;

fn view_for_commit_link(commit:&GitCommitSignature) -> Html<Context, Model> {

    let commit_hash_short = commit.git_hash_short_hex_string().unwrap_or("commit".into());

    return match commit.git_hash_hex_string().clone() {
        Some(commit_hash) => {
            html! {
                <a  class="commit-hash-link",
                    target="_blank",
                    href={ format!("https://github.com/search?q={}&type=Commits", commit_hash) },>
                    { format!("[commit {}] ", commit_hash_short.clone())}
                </a>
            }

        }
        None => {
            html! {
                <span class="commit-hash",>
                    { format!("[{}] ", commit_hash_short)}
                </span>
            }
        }
    };
}

pub fn view_for_logs_log_item(log:&LogByUser) -> Html<Context, Model> {
    let mut tokens = log.member_email.split("@");
    let (user, domain) = (tokens.next(), tokens.next());

    match log.log.body {
        LogBody::Ssh(ref signature) => {
            let host = signature.host_authorization.as_ref().map(|h| h.host.clone()).unwrap_or("unknown host".into());

            let (result_string, result_class)  = match signature.result {
                SSHSignatureResult::Signature(_) => {
                    ("✔", "audit-log-result-success")
                }
                _ => { ("✘", "audit-log-result-fail") }
            };

            html! {
                <tr>
                    <td>
                        <span class=("ssh", "badge"),>
                            {"SSH login"}
                        </span>
                    </td>
                    <td class={ result_class },>
                        { result_string }
                    </td>
                    <td class="log-body-info",>
                        { &signature.user }{" @ "}<span class="blue",> {host}</span>
                    </td>
                    <td>
                        <span class="blue",>
                            {user.unwrap_or("--")}
                        </span>
                        {"@"}{domain.unwrap_or("--")}
                    </td>

                    <td>{log.log.unix_seconds.time_ago()}</td>
                </tr>
            }
        },
        LogBody::GitCommit(ref commit) => {
            let message = &commit.message_string.clone().unwrap_or("unknown".into());


            let (result_string, result_class)  = match commit.result {
                GitSignatureResult::Signature(_) => {
                    ("✔", "audit-log-result-success")
                }
                _ => { ("✘", "audit-log-result-fail") }
            };

            html! {
                <tr>
                    <td>
                        <span class=("sign", "badge"),>
                            {"Git Sign"}
                        </span>
                    </td>
                    <td class={ result_class },>
                        { result_string }
                    </td>
                    <td class="log-body-info",>
                        { view_for_commit_link(commit) }{ message }
                    </td>
                    <td>
                        <span class="blue",>
                            { user.unwrap_or("--") }
                        </span>
                        {"@"}{ domain.unwrap_or("--") }
                    </td>
                    <td> <span class="log-timestamp",>{ log.log.unix_seconds.time_ago() } </span></td>
                </tr>
            }
        },
        logs::LogBody::GitTag(ref tag) => {
            let tag_string = &tag.tag;
            let message = &tag.message_string.clone().unwrap_or("unknown".into());

            let (result_string, result_class)  = match &tag.result {
                &GitSignatureResult::Signature(_) => {
                    ("✔", "audit-log-result-success")
                }
                _ => { ("✘", "audit-log-result-fail") }
            };

            html! {
                    <tr>
                        <td>
                            <span class=("sign", "badge"),>
                                {"Git Sign"}
                            </span>
                        </td>
                        <td class={ result_class },>
                            { result_string }
                        </td>
                    <td class="log-body-info",>
                            {format!("[Tag \"{}\"] {}", tag_string, message)}
                        </td>
                        <td>
                            <span class="blue",>
                                {user.unwrap_or("--")}
                            </span>
                            {"@"}{domain.unwrap_or("--")}
                        </td>
                        <td>{log.log.unix_seconds.time_ago()}</td>
                    </tr>
            }
        }
    }
}
