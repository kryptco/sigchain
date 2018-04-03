use super::*;
use diesel::prelude::*;
use diesel::{insert_into, delete};
use diesel::associations::HasTable;

#[derive(Identifiable, Queryable, Insertable, Debug, Clone)]
#[table_name="read_tokens"]
#[primary_key(team_public_key)]
pub struct ReadToken {
    pub team_public_key: Vec<u8>,
    pub token: Vec<u8>,
    pub reader_key_pair: Vec<u8>,
}

impl ReadToken {
    pub fn find(conn: &DBConnection, team_public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find(team_public_key).get_result(conn)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn delete(conn: &DBConnection, team_public_key: &[u8]) -> QueryResult<usize> {
        delete(Self::table().find(team_public_key)).execute(conn)
    }
}

#[derive(Queryable, Insertable, Debug, Clone)]
#[table_name="current_team"]
pub struct CurrentTeam {
    pub team_checkpoint: Vec<u8>,
    pub sign_key_pair: Option<Vec<u8>>,
    pub box_key_pair: Option<Vec<u8>>,
}

impl CurrentTeam {
    pub fn find(conn: &DBConnection) -> QueryResult<Self> {
        current_team::table.first(conn)
    }
    pub fn set(&self, conn: &DBConnection) -> QueryResult<usize> {
        Self::delete(conn)?;
        insert_into(current_team::table).values(self).execute(conn)
    }
    pub fn delete(conn: &DBConnection) -> QueryResult<()> {
        delete(current_team::table).execute(conn)?;
        delete(current_wrapped_keys::table).execute(conn)?;
        delete(queued_logs::table).execute(conn)?;
        Ok(())
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name="logs"]
pub struct Log {
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub log_json: String,
    pub unix_seconds: i64,
}

#[derive(Queryable, Debug, Clone)]
pub struct LogWithId {
    pub id: i64,
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub log_json: String,
    pub unix_seconds: i64,
}

impl Log {
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(logs::table).values(self).execute(conn)
    }
    pub fn all(conn: &TeamDBConnection, limit: Option<i64>) -> QueryResult<Vec<LogWithId>> {
        let sorted = logs::table.filter(logs::team_public_key.eq(conn.team))
            .order(logs::id.desc());

        match limit {
            Some(lim) => sorted.limit(lim).get_results(conn.conn),
            None => sorted.get_results(conn.conn)
        }
    }

    pub fn after(conn: &TeamDBConnection, id: Option<i64>, limit: Option<i64>) -> QueryResult<Vec<LogWithId>> {
        let the_id = match id {
            Some(the_id) => { the_id }
            _ => { return Log::all(conn, limit); }
        };

        let sorted = logs::table.filter(logs::team_public_key.eq(conn.team))
            .filter(logs::id.gt(the_id))
            .order(logs::id.desc());

        match limit {
            Some(lim) => sorted.limit(lim).get_results(conn.conn),
            None => sorted.get_results(conn.conn)
        }
    }

    pub fn for_member(conn: &TeamDBConnection, member_public_key: &[u8]) -> QueryResult<Vec<LogWithId>> {
        logs::table.filter(logs::team_public_key.eq(conn.team))
            .filter(logs::member_public_key.eq(member_public_key)).get_results(conn.conn)
    }
}

#[derive(Queryable, Insertable, Debug, Clone, Identifiable)]
#[table_name="current_wrapped_keys"]
#[primary_key(destination_public_key)]
pub struct CurrentWrappedKey {
    pub destination_public_key: Vec<u8>,
}

impl CurrentWrappedKey {
    pub fn all(conn: &DBConnection) -> QueryResult<Vec<Self>> {
        Self::table().get_results(conn)
    }
    pub fn clear(conn: &DBConnection) -> QueryResult<()> {
        delete(current_wrapped_keys::table).execute(conn)?;
        Ok(())
    }
    pub fn set(conn: &DBConnection, wrapped_keys: &[CurrentWrappedKey]) -> QueryResult<()> {
        Self::clear(conn)?;
        Self::add(conn, wrapped_keys)
    }
    pub fn add(conn: &DBConnection, wrapped_keys: &[CurrentWrappedKey]) -> QueryResult<()> {
        for wrapped_key in wrapped_keys {
            // Don't error if a client wraps a key twice in one epoch
            if current_wrapped_keys::table.find(wrapped_key.id()).first::<Self>(conn).optional()?.is_none() {
                insert_into(Self::table()).values(wrapped_key).execute(conn)?;
            }
        }
        Ok(())
    }
}

#[derive(Queryable, Debug, Clone, Identifiable)]
#[table_name="queued_logs"]
#[primary_key(id)]
pub struct QueuedLog {
    pub id: i64,
    pub log_json: Vec<u8>,
}

#[derive(Debug, Clone, Insertable)]
#[table_name="queued_logs"]
pub struct NewQueuedLog {
    pub log_json: Vec<u8>,
}

impl QueuedLog {
    pub fn next(conn: &DBConnection) -> QueryResult<Self> {
        Self::table().order(queued_logs::id.asc()).limit(1).first(conn)
    }
    pub fn remove(&self, conn: &DBConnection) -> QueryResult<()> {
        delete(Self::table().find(self.id)).execute(conn)?;
        Ok(())
    }
    pub fn clear(conn: &DBConnection) -> QueryResult<()> {
        delete(Self::table()).execute(conn)?;
        Ok(())
    }
    pub fn all(conn: &DBConnection) -> QueryResult<Vec<Self>> {
        Self::table().get_results(conn)
    }
    pub fn add(conn: &DBConnection, new_log: &NewQueuedLog) -> QueryResult<()> {
        insert_into(Self::table()).values(new_log).execute(conn)?;
        Ok(())
    }
    pub fn any(conn: &DBConnection) -> QueryResult<bool> {
        Self::next(conn).optional().map(|o| o.is_some())
    }
}
