use {Result, sigchain_core::errors::Error};
use std::process::*;

use crypto;
use base64;
use krd_client;
use client::try_with_delegated_network_cli;
use sigchain_core::protocol::Identity;
use client::traits::DBConnect;
use colored::Colorize;
use sigchain_core::util::text;

use std::io::prelude::*;

use clap::{App, Arg, SubCommand, ArgMatches};

fn build_kr_cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("kr")
        .subcommand(SubCommand::with_name("add")
            .arg(Arg::with_name("server")
                .long("server")
                .short("s")
                .takes_value(true)
                .required(true)
                .help("user@server or SSH alias to add public keys to"))
            .arg(Arg::with_name("port")
                .short("p")
                .takes_value(true)
                .help("Override default port of server"))
            .arg(Arg::with_name("member")
                .short("m")
                .multiple(true)
                .number_of_values(1)
                .use_delimiter(true)
                .takes_value(true)
                .help("Member email to add, can be specified multiple times"))
        )
        .subcommand(SubCommand::with_name("list")
            .visible_alias("ls")
            .arg(Arg::with_name("server")
                .long("server")
                .short("s")
                .takes_value(true)
                .required(true)
                .help("user@server or SSH alias to query"))
            .arg(Arg::with_name("port")
                .short("p")
                .takes_value(true)
                .help("Override default port of server"))
            .arg(Arg::with_name("ignore-unknown")
                 .long("ignore-unknown")
                 .help("Do not print public keys that do not correspond to any team member"))
        )
        .subcommand(SubCommand::with_name("remove")
            .visible_alias("rm")
            .arg(Arg::with_name("server")
                .long("server")
                .short("s")
                .takes_value(true)
                .required(true)
                .help("user@server or SSH alias to add public keys to"))
            .arg(Arg::with_name("port")
                .short("p")
                .takes_value(true)
                .help("Override default port of server"))
            // TODO: remove unknown keys using two SSH invocations and multiplexing
            //.arg(Arg::with_name("unknown")
                //.long("unknown")
                //.help("Remove public keys not associated with any team member"))
            .arg(Arg::with_name("member")
                .short("m")
                .multiple(true)
                .number_of_values(1)
                .use_delimiter(true)
                .takes_value(true)
                .help("Member email to add, can be specified multiple times"))
        )
}

fn parse_server_and_port(app: &ArgMatches) -> Result<(String, Option<u16>)> {
    let server = app.value_of("server").ok_or("server required")?;
    let port = {
        if let Some(port) = app.value_of("port") {
            Some(port.parse::<u16>()?)
        } else {
            None
        }
    };
    Ok((server.into(), port))
}

pub fn list_cli_command(args: &[&str]) -> Result<()> {
    let app = build_kr_cli_app().get_matches_from(args);

    if let Some(app) = app.subcommand_matches("list") {
        let (ref server, port) = parse_server_and_port(&app)?;
        let mut child = setup_ssh_controlmaster(server, port)?;
        defer!{{ let _ = child.wait(); }};
        query_and_print_acl(server, port, app.is_present("ignore-unknown"), true)?;
    } else {
        bail!("Invalid kr command")
    }

    Ok(())
}

fn query_and_print_acl(server: &str, port: Option<u16>, ignore_unknown: bool, update_team_blocks: bool) -> Result<()> {
    let public_keys = list_keys(server, port)?;

    let (matched_members, unmatched_keys) = try_with_delegated_network_cli(update_team_blocks, |client| -> Result<(Vec<Identity>, Vec<Vec<u8>>)> {
        use std::collections::HashMap;
        use std::iter::FromIterator;
        use client::traits::DBConnect;

        let members = client.get_active_members()?;
        let members_by_ssh_pk = HashMap::<Vec<u8>, Identity>::from_iter(
            members.into_iter().map(|i| (i.ssh_public_key.clone(), i))
        );

        let mut matched_members = vec![];
        let mut unmatched_keys = vec![];
        for pk in public_keys.iter() {
            match members_by_ssh_pk.get(pk) {
                Some(member) => matched_members.push(member.clone()),
                None => unmatched_keys.push(pk.clone()),
            };
        }

        use std::cmp::Ord;
        matched_members.sort_by(|a, b| String::cmp(&a.email, &b.email));
        matched_members.dedup_by(|a, b| a.public_key == b.public_key);

        Ok((matched_members, unmatched_keys))
    })?;

    if matched_members.len() > 0 {
        eprintln!("{}", format!("{} team members have access:", matched_members.len()).bright_green());
        for matched_member in matched_members {
            println!("{}", matched_member.email);
        }
    } else {
        eprintln!("No team members have access");
    }

    if unmatched_keys.len() > 0 && !ignore_unknown {
        eprintln!("{}", format!("\n{} unknown keys were found:", unmatched_keys.len()).bright_yellow());
        for unmatched_key in unmatched_keys {
            eprintln!("{}",
                      text::ellipsize_center(
                          &super::ssh_public_key_wire_string(&unmatched_key).unwrap_or("invalid key".to_string()),
                          80
                      ));
        }
    }

    Ok(())
}

pub fn add_cli_command(args: &[&str]) -> Result<()> {
    let app = build_kr_cli_app().get_matches_from(args);

    if let Some(app) = app.subcommand_matches("add") {
        let (ref server, port) = parse_server_and_port(&app)?;
        let mut child = setup_ssh_controlmaster(server, port)?;
        defer!{{ let _ = child.wait(); }};
        check_sshd_config(server, port)?;

        let keys = {
            if let Some(member_emails) = app.values_of("member") {
                let member_emails = member_emails.collect::<Vec<_>>();
                try_with_delegated_network_cli(true, |client| -> Result<Vec<Vec<u8>>> {
                    let mut member_pks = vec![];
                    for member_email in &member_emails {
                        member_pks.push(client.get_active_member_by_email(member_email)?.ssh_public_key);
                    }
                    Ok(member_pks)
                })?
            } else {
                vec![krd_client::daemon_me_request()?.ssh_wire_public_key]
            }
        };

        add_keys(keys, server, port)?;
        eprintln!();
        query_and_print_acl(server, port, true, false)?;
    } else {
        bail!("Invalid kr command")
    }
    Ok(())
}

pub fn rm_cli_command(args: &[&str]) -> Result<()> {
    let app = build_kr_cli_app().get_matches_from(args);

    if let Some(app) = app.subcommand_matches("remove") {
        let (ref server, port) = parse_server_and_port(&app)?;
        let mut child = setup_ssh_controlmaster(server, port)?;
        defer!{{ let _ = child.wait(); }};
        check_sshd_config(server, port)?;

        let keys = {
            if let Some(member_emails) = app.values_of("member") {
                let member_emails = member_emails.collect::<Vec<_>>();
                try_with_delegated_network_cli(true, |client| -> Result<Vec<Vec<u8>>> {
                    let mut member_pks = vec![];
                    for member_email in &member_emails {
                        // Remove current and any removed members' keys with matching email
                        member_pks.extend(
                            client.get_active_and_removed_by_email(member_email)?.into_iter()
                                .map(|i| i.ssh_public_key).collect::<Vec<_>>()
                        );
                    }
                    Ok(member_pks)
                })?
            } else {
                bail!("No public keys to remove");
            }
        };

        rm_keys(keys, server, port)?;
        eprintln!();
        query_and_print_acl(server, port, true, false)?;
    } else {
        bail!("Invalid kr command")
    }
    Ok(())
}

pub fn list_keys(server: &str, port: Option<u16>) -> Result<Vec<Vec<u8>>> {
    check_sshd_config(server, port)?;
    let mut args = ssh_args(server, port)?;
    args.push(list_keys_cmd());

    let output = Command::new("ssh")
        .stdin(Stdio::null()).stderr(Stdio::inherit()).stdout(Stdio::piped())
        .args(&args)
        .output()?;

    use std::str;
    if !output.status.success() {
        bail!("ssh command failed: {}", str::from_utf8(&output.stderr)?);
    }

    use std::collections::HashSet;
    use std::iter::FromIterator;
    let key_types = HashSet::<&&str>::from_iter(&[
        "ecdsa-sha2-nistp256", "ecdsa-sha2-nistp384", "ecdsa-sha2-nistp521", "ssh-ed25519", "ssh-dss", "ssh-rsa",
    ]);

    let mut invalid_lines : Vec<(usize, Error)> = vec![];
    let mut public_keys : Vec<Vec<u8>> = vec![];
    for (i, line) in output.stdout.lines().enumerate() {
        let valid_line = match line {
            Ok(line) => line,
            Err(e) => {
                invalid_lines.push((i, e.into()));
                continue;
            }
        };
        let valid_line = valid_line.trim();

        if valid_line.len() == 0 || valid_line.starts_with("#") {
            continue;
        }

        // Find public key as token after key type
        let tokens = valid_line.split_whitespace().collect::<Vec<_>>();
        let (token_idx, _) = match tokens.iter().enumerate()
            .filter(|&(_, token)| key_types.contains(token)).next() {
            Some(t) => t,
            None => {
                invalid_lines.push((i, "no public key type found".into()));
                continue;
            }
        };

        let key_b64 = match tokens.get(token_idx + 1) {
            Some(key_b64) => key_b64,
            None => {
                invalid_lines.push((i, "no public key found after key type".into()));
                continue;
            }
        };

        let key_bytes = match base64::decode(&key_b64) {
            Ok(bytes) => bytes,
            Err(e) => {
                invalid_lines.push((i, format!("failed to parse key bytes: {}", e).into()));
                continue
            }
        };
        //TODO: validate ssh-wire format here instead of in caller
        public_keys.push(key_bytes);
    }

    //TODO: print found errors in authorized_keys file

    Ok(public_keys)
}

pub fn add_keys(wire_format_keys: Vec<Vec<u8>>, server: &str, port: Option<u16>) -> Result<()> {
    let mut args = ssh_args(server, port)?;
    args.push(add_keys_cmd()?);

    if wire_format_keys.is_empty() {
        bail!("No public keys to add");
    }

    let mut stdin = Vec::<u8>::new();
    for wire_format_key in &wire_format_keys {
        stdin.extend(format!("{}\n", super::ssh_public_key_wire_string(wire_format_key)?).as_bytes());
    }

    let mut child = Command::new("ssh")
        .stdin(Stdio::piped()).stderr(Stdio::inherit()).stdout(Stdio::inherit())
        .args(&args)
        .spawn()?;
    if let Some(ref mut child_stdin) = child.stdin {
        child_stdin.write_all(&stdin)?;
    }

    if !child.wait()?.success() {
        bail!("ssh command failed");
    }
    Ok(())
}

pub fn rm_keys(wire_format_keys: Vec<Vec<u8>>, server: &str, port: Option<u16>) -> Result<()> {
    let mut args = ssh_args(server, port)?;
    args.push(rm_keys_cmd()?);

    if wire_format_keys.is_empty() {
        bail!("No public keys to remove");
    }

    let mut stdin = Vec::<u8>::new();
    for wire_format_key in &wire_format_keys {
        stdin.extend(format!("{}\n", super::ssh_public_key_wire_string(wire_format_key)?).as_bytes());
    }

    let mut child = Command::new("ssh")
        .stdin(Stdio::piped()).stderr(Stdio::inherit()).stdout(Stdio::inherit())
        .args(&args)
        .spawn()?;
    if let Some(ref mut child_stdin) = child.stdin {
        child_stdin.write_all(&stdin)?;
    }

    if !child.wait()?.success() {
        bail!("ssh command failed");
    }
    Ok(())
}

fn setup_ssh_controlmaster(server: &str, port: Option<u16>) -> Result<Child> {
    use std::fs;
    use std::env;
    let controlmaster_dir = format!("{}/.ssh/controlmasters/", env::var("HOME")?);
    fs::create_dir_all(&controlmaster_dir)?;
    if !Command::new("chmod").arg("0700").arg(&controlmaster_dir).status()?.success() {
        bail!("Failed to chmod ~/.ssh/controlmasters");
    }

    let mut args = ssh_args(server, port)?;
    args.push("bash -c \"echo connected; bash\"".to_string());

    let mut child = Command::new("ssh")
        .stdin(Stdio::piped()).stderr(Stdio::inherit()).stdout(Stdio::piped())
        .args(&args)
        .spawn()?;
    if let Some(ref mut child_stdout) = child.stdout {
        child_stdout.read_exact(&mut [0u8; 1])?;
    }
    Ok(child)
}

fn ssh_args(server: &str, port: Option<u16>) -> Result<Vec<String>> {
    use std::env;
    let mut args = vec![
        "-S".to_string(),
        format!("{}/.ssh/controlmasters/%C", env::var("HOME")?),
        "-o ControlMaster=auto".to_string(),
        server.to_string(),
    ];
    if let Some(port) = port {
        args.push(format!("-p {}", port))
    }
    Ok(args)
}

fn check_sshd_config(server: &str, port: Option<u16>) -> Result<()> {
    let mut args = ssh_args(server, port)?;
    args.push(check_sshd_config_cmd()?);

    let status = Command::new("ssh")
        .stdin(Stdio::null()).stderr(Stdio::inherit()).stdout(Stdio::inherit())
        .args(&args)
        .status()?;

    if !status.success() {
        bail!("Server incompatible with `kr` access control");
    }

    Ok(())
}

fn add_keys_cmd() -> Result<String> {
    // https://unix.stackexchange.com/questions/275794/iterating-over-multiple-line-string-stored-in-variable
    let nonce_filename = base64::encode_config(&crypto::random_nonce()?, base64::URL_SAFE);
    Ok(format!("bash -c 'keys=$(</dev/stdin); num_added=0; mkdir -m 700 -p ~/.ssh && touch ~/.ssh/authorized_keys \
    && cp ~/.ssh/authorized_keys ~/.ssh/{nonce_filename} \
    && echo \"$keys\" | ( while IFS= read -r key ; do \
        {{ grep \"$key\" ~/.ssh/{nonce_filename} 2>/dev/null 1>/dev/null || {{ echo \"Added `echo $key | cut -c-40`...`echo $key | rev | cut -c-40 | rev`\" 1>&2 && false; }} }} \
        || {{ echo $key >> ~/.ssh/{nonce_filename}; }} ; \
    done \
    && mv ~/.ssh/{nonce_filename} ~/.ssh/authorized_keys && \
    chmod 600 ~/.ssh/authorized_keys ) \
    '", nonce_filename = nonce_filename))
}

fn list_keys_cmd() -> String {
    format!("sh -c 'cat ~/.ssh/authorized_keys || {{ echo \"Could not read ~/.ssh/authorized_keys\" 1>&2; exit 1; }}'")
}

fn rm_keys_cmd() -> Result<String> {
    let nonce_filename = base64::encode_config(&crypto::random_nonce()?, base64::URL_SAFE);
    Ok(format!("bash -c 'keys=$(</dev/stdin); num_removed=0; mkdir -m 700 -p ~/.ssh && touch ~/.ssh/authorized_keys \
    && cp ~/.ssh/authorized_keys ~/.ssh/{nonce_filename} \
    && echo \"$keys\" | ( while IFS= read -r key ; do \
        {{ {{ grep \"$key\" ~/.ssh/{nonce_filename} 1>/dev/null 2>/dev/null && num_removed=$((num_removed+1)); }} || true; }} && \
        grep -v \"$key\" ~/.ssh/{nonce_filename} > ~/.ssh/{nonce_filename}.rm && mv ~/.ssh/{nonce_filename}.rm ~/.ssh/{nonce_filename}; \
    done \
    && mv ~/.ssh/{nonce_filename} ~/.ssh/authorized_keys
    chmod 600 ~/.ssh/authorized_keys \
    && echo \"$num_removed public key(s) removed\" 1>&2 ) \
    '", nonce_filename = nonce_filename))
}

fn check_sshd_config_cmd() -> Result<String> {
    let nonce_filename = base64::encode_config(&crypto::random_nonce()?, base64::URL_SAFE);
    // Generate temp key so that sshd -T succeeds
    Ok(format!("bash -c 'ssh-keygen -t ed25519 -f /tmp/{nonce_filename} -N \"\" &>/dev/null &&\
            config=`sshd -T -h /tmp/{nonce_filename} -C user=$USER,host=,addr= -f /etc/ssh/sshd_config`; \
            rm /tmp/{nonce_filename}{{,.pub}} ; \
            echo \"$config\" | grep \"authorizedkeysfile \\(%h/\\)\\?.ssh/authorized_keys\" &>/dev/null\
            || {{ echo Server does not use .ssh/authorized_keys for access control. && exit 1; }}\
    '", nonce_filename = nonce_filename))
}
