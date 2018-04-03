#![recursion_limit="128"]

#[macro_use] pub extern crate log;
pub extern crate env_logger;
#[macro_use] extern crate scopeguard;

pub extern crate base64;

#[macro_use] extern crate serde_derive;
pub extern crate serde;
extern crate serde_json;

extern crate itertools;
extern crate chrono;

#[macro_use]
extern crate lazy_static;

pub use std::io::Write;

extern crate dotenv;
pub use dotenv::dotenv;
pub use std::env;

pub use std::str::FromStr;

#[macro_use] extern crate error_chain;

#[cfg(feature = "reqwest")]
extern crate reqwest;

extern crate url;

#[macro_export]
macro_rules! success(
    ($arg:expr) => { {
        return Ok(serde_json::to_string(&Response::Success($arg))?);
    } }
);

#[macro_export]
macro_rules! success_string(
    ($arg:expr) => { {
        serde_json::to_string(&Response::Success($arg))?
    } }
);

#[macro_use]
pub extern crate sigchain_core;
use sigchain_core::errors;
pub use sigchain_core::protocol;
pub use sigchain_core::protocol::*;
pub use sigchain_core::diesel::prelude::*;

pub use sigchain_core::util::*;
#[cfg(target_os = "android")]
#[macro_use]
pub use sigchain_core::util::android_timing::*;

pub mod encoding;
pub use self::encoding::*;
pub use sigchain_core::db;
pub use db::DBConnection;
pub mod enclave_protocol;
pub use self::enclave_protocol::*;

#[cfg(feature = "network_client")]
pub mod krd_client;
#[cfg(feature = "network_client")]
pub use self::krd_client::*;

#[cfg(feature = "network_client")]
pub mod block_validation;

#[cfg(target_os="android")]
pub mod lib_android;

use errors::Result;
extern crate time;

extern crate colored;
#[allow(unused_imports)]
use colored::Colorize;

pub mod client;
#[allow(unused_imports)]
pub use self::client::*;
pub use sigchain_core::crypto;
#[allow(unused_imports)]
pub use sigchain_core::crypto::*;
pub use sigchain_core::sha256;

pub use sigchain_core::sshwire;
pub mod ssh;
pub use self::ssh::*;

pub use sigchain_core::pgp::*;

pub mod notification;
pub use notification::*;
