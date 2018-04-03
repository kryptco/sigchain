extern crate dashboard_middleware;
extern crate base64;

use std::env;
use std::sync::mpsc::{self};

pub fn main() {
    env::set_var("RUST_LOG", "info");
    let serve_result = dashboard_middleware::serve_dashboard(false);


    match serve_result {
        Ok(params) => {
            println!("Dashboard middleware started successfully.");
            println!("URL: http://localhost:{}/#{}", params.port, base64::encode_config(&params.token, base64::URL_SAFE));

            let (_tx, rx) = mpsc::channel::<()>();
            let _ = rx.recv();
        },
        Err(e) => { println!("Dashboard middleware failed to start: {:?}.", e); },
    };
}
