extern crate sigchain_client;
use sigchain_client::client;

extern crate sigchain_core;

#[macro_use]
extern crate error_chain;
extern crate serde_json;
extern crate open;
extern crate url;

extern crate colored;

extern crate itertools;

#[allow(unused_imports)]
pub use c_api::*;
pub mod c_api {
    extern crate dashboard_middleware;

    use sigchain_core::errors::{Result, Error};

    use sigchain_client::*;
    use std::str::from_utf8;
    use std::slice::from_raw_parts;

    use serde_json;
    use open;

    #[allow(unused_imports)]
    use colored::Colorize;

    use super::client::DelegatedNetworkClient;
    use super::client::traits::*;

    #[no_mangle]
    pub extern "C" fn create_indirect_emails_invite(emails_ptr: *const u8, emails_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            // comma separated
            let emails_string:String = unsafe{ from_utf8(from_raw_parts(emails_ptr, emails_len))?.into() };
            let emails:Vec<String> = emails_string.split(",").map(|s| s.into()).collect();
            let invite:String = client.create_invite(IndirectInvitationRestriction::Emails(emails))?;

            let team_name = client.get_team_info()?.name;
            print_invite_text(team_name, invite);
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn create_indirect_domain_invite(domain_ptr: *const u8, domain_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let domain:String = unsafe{ from_utf8(from_raw_parts(domain_ptr, domain_len))?.into() };
            let invite:String =  client.create_invite(IndirectInvitationRestriction::Domain(domain))?;
            let team_name = client.get_team_info()?.name;
            print_invite_text(team_name, invite);
            Ok(())
        });
    }


    #[no_mangle]
    pub extern "C" fn cancel_invite() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            client.cancel_invite()?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn remove_member(email_ptr: *const u8, email_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let email = unsafe{ from_utf8(from_raw_parts(email_ptr, email_len))? };
            client.remove_member(email)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn is_admin() -> bool {
        // Update team blocks
        do_with_delegated_network_cli(|_client| -> Result<()> { Ok(()) });

        if let Ok(client) = DelegatedNetworkClient::for_current_team() {
            match client.is_admin() {
                Ok(admin) => return admin,
                _ => return false,
            }
        }
        false
    }

    #[no_mangle]
    pub extern "C" fn add_admin(email_ptr: *const u8, email_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let email = unsafe{ from_utf8(from_raw_parts(email_ptr, email_len))? };
            client.add_admin(email)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn remove_admin(email_ptr: *const u8, email_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let email = unsafe{ from_utf8(from_raw_parts(email_ptr, email_len))? };
            client.remove_admin(email)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn get_admins() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            for (idx, admin) in client.get_admins()?.iter().enumerate() {
                eprint!("{}", format!("{}. ", idx + 1).green());
                println!("{}", admin.email);
            }
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn get_policy() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            match client.get_policy()?.temporary_approval_seconds {
                Some(approval_window) => {
                    eprint!("{}", "Auto-approval window is ".green());
                    if approval_window == 0 {
                        eprintln!("{}", "disabled".green());
                    } else {
                        eprintln!("{} {} minutes", "set to".green(), approval_window / 60);
                    }
                },
                None => eprintln!("{}", "Auto-approval window is unrestricted".green()),
            }
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn set_policy(approval_window: *const i64) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let policy = if approval_window.is_null() {
                Policy {
                    temporary_approval_seconds: None,
                }
            } else {
                Policy {
                    temporary_approval_seconds: Some(unsafe{ *approval_window }),
                }
            };

            client.set_policy(policy.clone())?;

            match policy.temporary_approval_seconds {
                Some(seconds) => {
                    if seconds == 0 {
                        eprintln!("{}", "Success! Team’s temporary auto-approval is now disabled ✔".green());
                    } else {
                        eprint!("{}", "Success! Team’s temporary auto-approval window is now set to ".green());
                        eprint!("{} minutes ", seconds / 60);
                        eprintln!("{}", "✔".green());
                    }
                }
                None => {
                    eprintln!("{}", "Success! Team’s temporary auto-approval window is unrestricted ✔".green());
                }
            };

            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn set_team_name(name_ptr: *const u8, name_len: usize) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let name = from_utf8(unsafe{ from_raw_parts(name_ptr, name_len) })?;
            client.set_team_info(TeamInfo{
                name: name.into(),
            })?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn get_members(email_ptr: *const u8, email_len: usize,
                                  print_ssh_pubkey: bool, print_pgp_pubkey: bool) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let email: Option<&str> = match email_ptr.is_null() {
                true => None,
                false => Some(from_utf8(
                    unsafe{ from_raw_parts(email_ptr, email_len) }
                )?),
            };

            let members = match email {
                Some(e) => vec![client.get_active_member_by_email(e)?],
                None => client.get_active_members()?,
            };

            match email {
                Some(e) => eprintln!("Found team member with email {}", format!("{}", e).yellow()),
                None => eprintln!("Team has {}", format!("{} members", members.len()).green()),
            }

            if print_ssh_pubkey && print_pgp_pubkey {
                eprintln!("Printing SSH/PGP Keys:");
            } else if print_ssh_pubkey {
                eprintln!("Printing SSH Keys:");
            } else if print_pgp_pubkey {
                eprintln!("Printing PGP Keys:");
            }
            eprintln!();

            for (idx, identity) in members.iter().enumerate() {
                let header = format!("{}. {}", idx + 1, identity.email).green();
                eprintln!("{}", header);
                if print_ssh_pubkey {
                    if let Ok(ssh_pubkey) = ssh_public_key_wire_string(&identity.ssh_public_key) {
                        println!("{} {}", ssh_pubkey, identity.email);
                    }
                }
                if print_pgp_pubkey {
                    if let Ok(pgp_pubkey) = pgp_public_key_ascii_armor_string(&identity.pgp_public_key) {
                        println!("{}", pgp_pubkey);
                    }
                }
                if print_ssh_pubkey || print_pgp_pubkey {
                    eprintln!();
                }
            }

            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn pin_host_key(
        host_ptr: *const u8, host_len: usize,
        public_key_ptr: *const u8, public_key_len: usize,
    ) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let host = from_utf8(unsafe{ from_raw_parts(host_ptr, host_len) })?;
            let public_key = unsafe{ from_raw_parts(public_key_ptr, public_key_len) };

            client.pin_host_key(host, public_key)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn pin_known_host_keys(
        host_ptr: *const u8, host_len: usize,
        update_from_server: bool,
    ) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let host_or_alias = from_utf8(unsafe{ from_raw_parts(host_ptr, host_len) })?;
            let host_port = map_ssh_alias_to_hostname_port(host_or_alias)?;

            if update_from_server {
                eprintln!("Updating known hosts for {}...", host_port);
                update_known_hosts(&host_or_alias).or_else::<Error, _>(
                    |e| bail!("Error updating known hosts: {}", e))?;
                eprintln!("Updated known hosts for {}", host_port);
            }
            let known_host_keys = local_host_keys(&host_port)?;

            let pinned_keys = client.get_pinned_host_keys(&host_port, false)?
                .into_iter().map(|pinned_key| pinned_key.public_key).collect::<Vec<_>>();
            for public_key in known_host_keys.clone() {
                let wire_string = ssh_public_key_wire_string(&public_key).unwrap_or(
                    base64::encode(&public_key));
                if pinned_keys.contains(&public_key) {
                    eprintln!("already pinned {}", wire_string);
                    continue;
                }

                client.pin_host_key(&host_port, &public_key)?;
                eprintln!(
                    "{} {} {} {}",
                    "Success! ".green(),
                    host_or_alias,
                    " is now pinned to public key ".green(),
                    wire_string
                );
            }

            let other_pinned_keys = pinned_keys
                .into_iter().filter(|key| !known_host_keys.contains(key)).collect::<Vec<Vec<u8>>>();
            if other_pinned_keys.len() > 0 {
                if prompt(
                    &format!("{} unknown keys are still pinned. Unpin them?", other_pinned_keys.len()).red()
                )? {
                    for other_pinned_key in other_pinned_keys {
                        client.unpin_host_key(&host_port, &other_pinned_key)?;
                        eprintln!("unpinned {}", ssh_public_key_wire_string(&other_pinned_key)?);
                    }
                }
            }

            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn get_pinned_host_keys(
        host_ptr: *const u8, host_len: usize,
        search: bool,
    ) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let host_or_alias = from_utf8(unsafe{ from_raw_parts(host_ptr, host_len) })?;
            let host_port = map_ssh_alias_to_hostname_port(host_or_alias)?;

            let mut pinned_keys = client.get_pinned_host_keys(&host_port, search)?;
            pinned_keys.sort_by_key(|pinned_key| pinned_key.host.clone());
            for pinned_key in pinned_keys.into_iter() {
                eprintln!("{} {}",
                          pinned_key.host,
                          ssh_public_key_wire_string(&pinned_key.public_key).unwrap_or("invalid key format".into()));
            }
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn get_all_pinned_host_keys() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let mut all_pinned_keys = client.get_all_pinned_host_keys()?;
            all_pinned_keys.sort_by_key(|pinned_key| pinned_key.host.clone());
            for pinned_key in all_pinned_keys.into_iter() {
                eprintln!("{} {}",
                          pinned_key.host,
                          ssh_public_key_wire_string(&pinned_key.public_key).unwrap_or("invalid key format".into()));
            }
            Ok(())
        });
    }


    #[no_mangle]
    pub extern "C" fn unpin_host_key(
        host_ptr: *const u8, host_len: usize,
        public_key_ptr: *const u8, public_key_len: usize,
    ) {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let host = from_utf8(unsafe{ from_raw_parts(host_ptr, host_len) })?;
            let public_key = unsafe{ from_raw_parts(public_key_ptr, public_key_len) };

            client.unpin_host_key(host, public_key)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn enable_logging() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            client.enable_logging()
        });
    }

    #[no_mangle]
    pub extern "C" fn update_team_logs() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            client.update_team_log_blocks()
        });
    }

    #[no_mangle]
    pub extern "C" fn view_logs() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            eprintln!("Fetching logs...");
            client.update_team_log_blocks()?;
            eprintln!("{}", format!("Done fetching logs ✔").green());
            eprintln!("Tailing last 100 logs:");
            eprintln!();

            let conn = &client.team_db_conn();

            let mut last_log_id:Option<i64> = None;

            let mut skip_slow_print = true;

            loop {
                use std::thread;
                use std::time::Duration;

                let log_result = db::Log::after(conn, last_log_id, if let Some(_) = last_log_id {
                    None
                } else {
                    Some(100)
                })?;
                match log_result.first().map(|db_log| db_log.id).clone() {
                    Some(id) => last_log_id = Some(id),
                    _ => {}
                };

                let logs: Vec<(String, logs::Log)> = log_result.iter().filter_map(|db_log| {
                    let log_object = match serde_json::from_str::<logs::Log>(&db_log.log_json) {
                        Ok(object) => { object }
                        Err(_) => { return None; }
                    };

                    let email = match db::Identity::find(conn, &db_log.member_public_key) {
                        Ok(identity) => { identity.email }
                        Err(_) => { return None; }
                    };

                    Some((email, log_object))
                }).collect();

                let mut sorted_logs = logs.clone();
                sorted_logs.sort_by_key(|log| log.1.unix_seconds);

                for email_log in sorted_logs {
                    let (email, log) = email_log;

                    use sigchain_core::time_util::TimeAgo;
                    use sigchain_core::git_hash::*;

                    let mut log_type:String;
                    let mut log_body_string:String;

                    match log.body {
                        LogBody::Ssh(ref signature) => {
                            let host:String = signature.clone().host_authorization.map(|h| h.host).unwrap_or("unknown host".into());

                            log_type = "SSH".into();
                            log_body_string = format!("{} @ {}", signature.user, host).yellow().to_string();

                        },
                        LogBody::GitCommit(ref commit) => {
                            let message:String = commit.clone().message_string.unwrap_or("unknown".into());

                            log_type = "Git".into();
                            log_body_string = format!("[{}] {}", commit.git_hash_short_hex_string().unwrap_or("?".into()), message.trim()).into();
                        },
                        logs::LogBody::GitTag(ref tag) => {
                            let cloned_tag = tag.clone();
                            let message:String = cloned_tag.message_string.unwrap_or("unknown".into());

                            log_type = "Git".into();
                            log_body_string = format!("Tag {}: {}", cloned_tag.tag, message).into();
                        }
                    };

                    let log_string = format!("{:18}    {:30}    Device: {:20}    {}",
                                             log.unix_seconds.full_timestamp(),
                                             email,
                                             log.session.device_name,
                                             log_body_string,
                                             );

                    if log.body.is_success() {
                        println!("{}", format!("[{}]\t✔\t{}", log_type, log_string).green());
                    } else {
                        println!("{}", format!("[{}]\t✘\t{}", log_type, log_string).red());
                    }

                    if !skip_slow_print {
                        thread::sleep(Duration::from_millis(200));
                    }
                }

                skip_slow_print = false;
                client.update_team_log_blocks()?;
                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    #[no_mangle]
    pub extern "C" fn open_billing() {
        do_with_delegated_network_cli(|client| -> Result<()> {
            let team_name = client.get_team_info()?.name;
            let aem = client.get_my_identity()?.email;

            let billing_url:String = client.public_key_and_checkpoint.server_endpoints.billing_url(
                &team_name,
                &client.public_key_and_checkpoint.team_public_key,
                &client.public_key_and_checkpoint.public_key,
                &aem,
            )?;
            eprintln!("Billing can be found at: {}", format!("{}", billing_url).magenta());
            if open::that(billing_url).is_ok() {
                eprintln!("Opening a browser...done");
            }

            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn serve_dashboard() {
        do_with_delegated_network_cli(|_client| -> Result<()> {
            dashboard_middleware::serve_dashboard(false)?;
            Ok(())
        });
    }

    #[no_mangle]
    pub extern "C" fn serve_dashboard_if_params_present() {
        let _ = dashboard_middleware::serve_dashboard(true);
    }

    fn print_invite_text(team_name:String, invite_link:String) {
        eprintln!();
        eprintln!("{}", format!("Link created! Send the following invitation to new members:").magenta());
        eprintln!();
        eprintln!("{}", format!("You're invited to join {} on Krypton!", team_name));
        eprintln!("{}", format!("Step 1. Install: https://get.krypt.co"));
        eprintln!("{}", format!("Step 2. Accept Invite: tap the link below on your phone or copy this message (or just the link) into Krypton."));
        eprintln!("{}", format!("{}", invite_link));
        eprintln!();
    }

    use ::std::os::raw::c_char;
    use ::std::process::exit;
    use sigchain_client;
    // Pass arguments from Go since the rust native library std::env::args returns an empty list on Linux
    #[no_mangle]
    pub extern "C" fn kr_add(args: *const *const c_char, num_args: usize) {
        let args = parse_arg_array_or_fatal(args, num_args);
        let _ = sigchain_client::ssh::add_cli_command(args.as_slice()).map_err(|e| eprintln!("Error adding keys: {}", e));
    }
    #[no_mangle]
    pub extern "C" fn kr_list(args: *const *const c_char, num_args: usize) {
        let args = parse_arg_array_or_fatal(args, num_args);
        let _ = sigchain_client::ssh::list_cli_command(args.as_slice()).map_err(|e| eprintln!("Error listing keys: {}", e));
    }
    #[no_mangle]
    pub extern "C" fn kr_rm(args: *const *const c_char, num_args: usize) {
        let args = parse_arg_array_or_fatal(args, num_args);
        let _ = sigchain_client::ssh::rm_cli_command(args.as_slice()).map_err(|e| eprintln!("Error removing keys: {}", e));
    }

    fn parse_arg_array_or_fatal<'a>(args: *const *const c_char, num_args: usize) -> Vec<&'a str> {
        use std::slice;
        use std::ffi::CStr;
        use itertools::process_results;

        let args_slice = unsafe{ slice::from_raw_parts(args, num_args) };
        let parsed_args = unsafe{ args_slice.into_iter().map(|ptr| CStr::from_ptr(*ptr)).map(CStr::to_str) };

        if let Ok(args) = process_results(parsed_args, |iter| iter.collect::<Vec<_>>()) {
            return args
        } else {
            eprintln!("Error parsing arguments");
            exit(1)
        }
    }
}
