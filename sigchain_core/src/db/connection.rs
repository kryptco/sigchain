use diesel;
use diesel::prelude::*;

use super::Result;
use dotenv;
use env;

#[cfg(feature = "pg")]
pub type DBConnection = diesel::pg::PgConnection;
#[cfg(feature = "sqlite")]
pub type DBConnection = diesel::sqlite::SqliteConnection;

pub fn establish_connection() -> Result<DBConnection> {
    time_fn!("establish_connection");
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let conn = DBConnection::establish(&database_url)?;

    super::embedded_migrations::run(&conn)?;
    Ok(conn)
}


pub struct TeamDBConnection<'a, 'b> {
    pub conn: &'a DBConnection,
    pub team: &'b [u8],
}
