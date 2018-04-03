#![recursion_limit="128"]

#[cfg(feature = "crypto")]
extern crate sodiumoxide;
#[cfg(feature = "crypto")]
pub use self::sodiumoxide::crypto::sign::ed25519;
#[cfg(feature = "crypto")]
pub use self::sodiumoxide::crypto::hash::sha256;
#[cfg(feature = "crypto")]
pub mod crypto;

pub extern crate chrono;

pub extern crate base64;

#[macro_use]
extern crate lazy_static;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

#[cfg(feature = "db")]
#[macro_use] pub extern crate diesel;
#[cfg(feature = "db")]
#[macro_use] pub extern crate diesel_migrations;
extern crate dotenv;
pub use dotenv::dotenv;
pub use std::env;
# [macro_use] extern crate error_chain;

#[macro_use] pub extern crate log;
#[macro_use] extern crate scopeguard;

#[cfg(feature = "reqwest")]
pub extern crate reqwest;
#[cfg(feature = "ssh-wire")]
pub extern crate sshwire;
#[cfg(feature = "hyper")]
pub extern crate hyper;
#[cfg(target_os = "android")]
pub extern crate jni;

extern crate url;

#[macro_use]
pub mod util;
pub use self::util::*;

#[cfg(feature = "db")]
pub mod db;

pub mod encoding;
pub use encoding::*;

pub mod protocol;
use protocol::*;

pub mod errors;
use errors::*;

pub mod pgp;

pub mod dashboard_protocol;
