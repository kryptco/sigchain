use Result;
use std::process::*;

use crypto;
use base64;
use krd_client;

use std::io::prelude::*;

use clap::{App, Arg, SubCommand};

pub fn add_cli_command() -> Result<()> {
    let app = App::new("kr")
        .subcommand(SubCommand::with_name("add")
            .arg(Arg::with_name("server")
                .required(true)
                .index(1)
                .help("user@server or SSH alias to add public keys to"))
            .arg(Arg::with_name("port")
                .short("p")
                .takes_value(true)
                .help("Override default port of server"))
            .arg(Arg::with_name("member")
                .short("m")
                .multiple(true)
                .takes_value(true)
                .help("Member email to add, can be specified multiple times"))
        ).get_matches();

    if let Some(app) = app.subcommand_matches("add") {
        let server = app.value_of("server").ok_or("server required")?;
        let port = {
            if let Some(port) = app.value_of("port") {
                Some(port.parse::<u16>()?)
            } else {
                None
            }
        };

        let keys = {
//        if app.num_occurrences("member") > 0 {
//            app.values_of("member").map(TODO)
//        }
            vec![krd_client::daemon_me_request()?.ssh_wire_public_key]
        };

        add_keys(keys, server, port)?;
    }
    Ok(())
}

pub fn add_keys(wire_format_keys: Vec<Vec<u8>>, server: &str, port: Option<u16>) -> Result<()> {
    let mut args = vec![server.to_string()];
    if let Some(port) = port {
        args.push(format!("-p {}", port))
    }
    args.push(add_keys_cmd()?);

    let mut stdin = Vec::<u8>::new();
    for wire_format_key in wire_format_keys {
        stdin.extend(format!("{}\n", super::ssh_public_key_wire_string(&wire_format_key)?).as_bytes());
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

fn add_keys_cmd() -> Result<String> {
    let nonce_filename = base64::encode_config(&crypto::random_nonce()?, base64::URL_SAFE);
    Ok(format!("sh -c 'read keys; mkdir -m 700 -p ~/.ssh && touch ~/.ssh/authorized_keys && \
    {{ grep \"$keys\" ~/.ssh/authorized_keys 2>/dev/null 1>/dev/null && echo Public key already has access; }} \
    || {{ cp ~/.ssh/authorized_keys ~/.ssh/{nonce_filename} && echo $keys >> ~/.ssh/{nonce_filename} && mv ~/.ssh/{nonce_filename} ~/.ssh/authorized_keys; }} ; \
    chmod 600 ~/.ssh/authorized_keys'", nonce_filename = nonce_filename))
}