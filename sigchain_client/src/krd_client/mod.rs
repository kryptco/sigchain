use {env, serde, errors, serde_json, enclave_protocol};
use super::Result;

extern crate hyperlocal;
use self::hyperlocal::{Uri, UnixConnector};

extern crate futures;
use self::futures::Stream;
use self::futures::Future;

pub extern crate hyper;
use self::hyper::{Client};

extern crate tokio_core;
use self::tokio_core::reactor::Core;

extern crate users;
use self::users::get_user_by_name;
use self::users::os::unix::UserExt;

use std::path::PathBuf;

use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::OpenOptions;
use std::fs;

pub fn kr_path() -> Result<PathBuf> {
    let home_path = env::home_dir();
    let mut kr_path = match home_path {
        Some(home_path) => { home_path }
        _ => {
            bail!("no home directory found");
        }
    };

    kr_path.push(".kr");

    fs::create_dir_all(&kr_path)?;
    Ok(kr_path)
}

fn daemon_control_path() -> Result<PathBuf> {
    let mut krd_control_sock = kr_path()?;
    krd_control_sock.push("krd.sock");
    return Ok(krd_control_sock);
}

fn notify_logs_dir_path() -> Result<PathBuf> {
    let mut notify_logs_path = kr_path()?;
    notify_logs_path.push("notify");
    return Ok(notify_logs_path);
}

fn notify_log_path(request_id: String) -> Result<PathBuf> {
    let mut log_path = notify_logs_dir_path()?;
    log_path.push(format!("krd-notify.log-[{}]", request_id));
    return Ok(log_path);
}

fn start_logger(request_id: String) -> Result<()>{
    let log_path = notify_log_path(request_id)?;
    let file = OpenOptions::new().read(true).write(true).create(true).open(log_path.as_path())?;
    let mut buf_reader = BufReader::new(file);

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(50));

            let mut line_buffer = String::new();
            let _ = match buf_reader.read_line(&mut line_buffer) {
                Ok(num_bytes) => { num_bytes }
                Err(_) => { continue; }
             };

            if line_buffer == "\n" || line_buffer == "" {
                continue;
            }

            eprintln!("{}", line_buffer.trim_right());
        }
    });
    Ok(())
}

fn daemon_control_request<R: serde::de::DeserializeOwned>(request: hyper::Request) -> Result<R> {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(UnixConnector::new(handle))
        .build(&core.handle());

    let work = client
        .request(request).from_err::<errors::Error>()
        .and_then(|res| {
            res.body().concat2().from_err().and_then(move |body| {
                use std::string::String;
                debug!("Response from krd: {}", String::from_utf8_lossy(&body));
                Ok(serde_json::from_slice::<R>(&body)?)
            })
        });

    Ok(core.run(work)?)
}

pub fn daemon_me_request_force_refresh() -> Result<enclave_protocol::Profile> {
    let request = hyper::Request::new(hyper::Method::Get, Uri::new(daemon_control_path()?, "/pair").into());
    debug!("Sending force refresh to krd");
    daemon_control_request::<enclave_protocol::Profile>(request)
}

pub fn daemon_enclave_control_request(enclave_request: &enclave_protocol::Request,
                                      should_read_notify_logs: bool) -> Result<enclave_protocol::Response> {

    // start the logger for acks if needed
    if should_read_notify_logs {
        start_logger(enclave_request.id.clone())?;
    }

    use colored::Colorize;

    // print the requesting indicator
    eprintln!("{}", format!("Krypton ▶ Requesting team operation from phone").cyan());

    let mut request = hyper::Request::new(hyper::Method::Put, Uri::new(daemon_control_path()?, "/enclave").into());
    let json_body = serde_json::to_string(&enclave_request)?;
    debug!("Sending enclave request: {}", json_body);
    request.set_body(json_body);


    let response_result = daemon_control_request::<enclave_protocol::Response>(request);
    // print the response result
    match response_result {
        Ok(ref response) => {
            match response.body.error() {
                None => { eprintln!("{}", format!("Krypton ▶ Success. Request Allowed ✔").green())}
                Some(err) => { eprintln!("{}", format!("Krypton ▶ Request failed: {} ✘", err.error).red()) }
            };
        }
        _=> {}
    }
    response_result
}

pub fn daemon_me_request() -> Result<enclave_protocol::Profile> {
    use enclave_protocol::*;

    let me_request = enclave_protocol::Request::new(RequestBody::MeRequest(MeRequest{ pgp_user_id: None }))?;

    let mut request = hyper::Request::new(hyper::Method::Get, Uri::new(daemon_control_path()?, "/enclave").into());
    let json_body = serde_json::to_string(&me_request)?;
    debug!("Sending me request to krd: {}", json_body);
    request.set_body(json_body);

    let response_result = daemon_control_request::<enclave_protocol::Response>(request)?;

    let profile = match response_result.body {
        ResponseBody::MeResponse(me_result) => {
            match me_result {
                enclave_protocol::Result::Success(me) => me.me,
                enclave_protocol::Result::Error(e) => bail!(e.error),
            }
        }
        _ => bail!("{:?}", "no me response returned"),
    };

    Ok(profile)
}
