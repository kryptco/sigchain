use super::{team};
use team::{MainChain};
use protocol::SignedMessage;
use logging;

use ::errors::{Result, Error};
pub use diesel::connection::Connection;


embed_migrations!();
pub fn run_migrations(conn: &DBConnection) -> Result<()> {
    Ok(embedded_migrations::run(conn)?)
}

use diesel;
pub fn uniqueness_to<E: Into<Error>>(e: E, r: ::errors::specified::ErrorKind) -> Error {
    use diesel::result::Error::DatabaseError;
    use ::errors::ErrorKind::*;
    let e = e.into();
    match &e {
        &::errors::Error(Diesel(DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)), _) => r.into(),
        _ => e,
    }
}

mod shared_schema;
use self::shared_schema::teams;
use self::shared_schema::blocks;
pub use self::shared_schema::log_blocks;
use self::shared_schema::team_memberships;
use self::shared_schema::indirect_invitations;
use self::shared_schema::direct_invitations;
use self::shared_schema::identities;
use self::shared_schema::pinned_host_keys;
use self::shared_schema::log_chains;

pub mod connection;
pub use self::connection::*;

pub mod models;
pub use self::models::*;

mod client_schema;
use self::client_schema::*;

pub mod client_models;
pub use self::client_models::*;
