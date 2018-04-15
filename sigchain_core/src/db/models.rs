use super::*;
use diesel::prelude::*;
use diesel::expression::dsl::{exists};
use diesel::{select, insert_into, update, delete};
use diesel::associations::HasTable;
use chrono;

use serde_json;

#[derive(Queryable, Insertable, Identifiable, Debug, Clone)]
#[primary_key(hash)]
#[table_name="blocks"]
pub struct Block {
    pub hash: Vec<u8>,
    pub last_block_hash: Option<Vec<u8>>,
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub operation: String, //  serialized Payload i.e. { write_block: ... }
    pub signature: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
}

impl Block {
    pub fn build(
        request: &SignedMessage,
        payload: &MainChain,
        team_public_key: Vec<u8>,
        ) -> Result<Block> {
        Self::build_(request, payload.last_block_hash(), team_public_key)
    }
    fn build_(
        request: &SignedMessage,
        last_block_hash: Option<Vec<u8>>,
        team_public_key: Vec<u8>,
    ) -> Result<Block> {
        Ok(Block{
            hash: request.payload_hash(),
            team_public_key,
            member_public_key: request.public_key.clone(),
            operation: request.message.clone(),
            signature: request.signature.clone().into(),
            last_block_hash,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn exists(conn: &DBConnection, hash: &[u8]) -> QueryResult<bool> {
        Self::find(conn, hash).optional().map(|opt_block| opt_block.is_some())
    }
    pub fn find(conn: &DBConnection, hash: &[u8]) -> QueryResult<Self> {
        Self::table().find(hash).first::<Self>(conn)
    }
    pub fn find_next(conn: &TeamDBConnection, last_block_hash: &Option<Vec<u8>>) -> QueryResult<Option<Self>> {
        let team_filter = Self::table().filter(blocks::team_public_key.eq(conn.team));
        match last_block_hash {
            &None =>
                team_filter.filter(blocks::last_block_hash.is_null())
                      .first::<Self>(conn.conn).optional(),
            &Some(ref last_block_hash) =>
                team_filter.filter(blocks::last_block_hash.eq(last_block_hash))
                      .first::<Self>(conn.conn).optional(),
        }
    }
    pub fn collect_all(conn: &TeamDBConnection) -> Result<Vec<Self>> {
        let mut blocks = vec![];
        let mut last_block_hash = None;
        while let Some(block) = Self::find_next(conn, &last_block_hash)? {
            last_block_hash = Some(block.hash.clone());
            blocks.push(block);
        }
        Ok(blocks)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn count(conn: &TeamDBConnection) -> Result<u64> {
        use self::blocks::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team)).count().get_result::<i64>(conn.conn)?.to_u64()
    }
}

#[derive(Queryable, Insertable, Identifiable, Debug, Clone)]
#[primary_key(hash)]
#[table_name="log_blocks"]
pub struct LogBlock {
    pub hash: Vec<u8>,
    pub last_block_hash: Option<Vec<u8>>,
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub operation: String, //  serialized Payload i.e. { write_block: ... }
    pub signature: Vec<u8>,
    created_at: chrono::NaiveDateTime,
}

impl LogBlock {
    pub fn build(
        request: &SignedMessage,
        payload: &logging::LogChain,
        team_public_key: Vec<u8>,
    ) -> Result<LogBlock> {
        let last_block_hash = match payload{
            &logging::LogChain::Create(_) => None,
            &logging::LogChain::Append(ref append_log_block) => Some(append_log_block.last_block_hash.clone()),
            &logging::LogChain::Read(_) => bail!("cannot store ReadLogBlocksRequest"),
        };
        Self::build_(request, last_block_hash, team_public_key)
    }

    fn build_(
        request: &SignedMessage,
        last_block_hash: Option<Vec<u8>>,
        team_public_key: Vec<u8>,
    ) -> Result<LogBlock> {
        Ok(LogBlock{
            hash: request.payload_hash(),
            team_public_key,
            member_public_key: request.public_key.clone(),
            operation: request.message.clone(),
            signature: request.signature.clone().into(),
            last_block_hash,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn exists(conn: &DBConnection, hash: &[u8]) -> QueryResult<bool> {
        select(exists(
            Self::table().find(hash)
        )).get_result(conn)
    }
    pub fn find(conn: &DBConnection, hash: &[u8]) -> QueryResult<Self> {
        Self::table().find(hash).first::<Self>(conn)
    }
    pub fn find_on_team(conn: &TeamDBConnection, hash: &[u8]) -> QueryResult<Self> {
        Self::table()
            .filter(log_blocks::team_public_key.eq(conn.team))
            .find(hash).first::<Self>(conn.conn)
    }
    pub fn find_for_member(conn: &DBConnection, member_public_key: &[u8], hash: &[u8]) -> QueryResult<Self> {
        Self::table()
            .filter(log_blocks::member_public_key.eq(member_public_key))
            .find(hash).first::<Self>(conn)
    }
    pub fn filter_by_member(conn: &TeamDBConnection, member_public_key: &[u8]) -> QueryResult<Vec<Self>> {
        Self::table()
            .filter(log_blocks::team_public_key.eq(conn.team))
            .filter(log_blocks::member_public_key.eq(member_public_key))
            .get_results(conn.conn)
    }
    pub fn find_next(conn: &TeamDBConnection, member_public_key: &[u8], last_block_hash: &Option<Vec<u8>>) -> QueryResult<Option<Self>> {
        let log_chain_filter = Self::table().filter(log_blocks::team_public_key.eq(conn.team))
            .filter(log_blocks::member_public_key.eq(member_public_key));
        match last_block_hash {
            &None =>
                log_chain_filter.filter(log_blocks::last_block_hash.is_null())
                           .first::<Self>(conn.conn).optional(),
            &Some(ref last_block_hash) =>
                log_chain_filter.filter(log_blocks::last_block_hash.eq(last_block_hash))
                           .first::<Self>(conn.conn).optional(),
        }
    }
    pub fn next_block_exists(conn: &TeamDBConnection, member_public_key: &[u8], last_block_hash: &Option<Vec<u8>>) -> QueryResult<bool> {
        Self::find_next(conn, member_public_key, last_block_hash).map(|opt_block| opt_block.is_some())
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn count_last_30_days(conn: &TeamDBConnection) -> Result<u64> {
        use self::log_blocks::dsl;
        use chrono;
        use std::ops::Sub;
        Self::table()
            .filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::created_at.gt(chrono::Utc::now().naive_utc().sub(chrono::Duration::days(30))))
            .count().get_result::<i64>(conn.conn)?.to_u64()
    }
}

#[derive(Debug, PartialEq, Clone, Queryable, Identifiable, Insertable, AsChangeset)]
#[table_name="team_memberships"]
#[primary_key(team_public_key, member_public_key)]
pub struct TeamMembership {
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub email: String,
    pub is_admin: bool,
}

impl TeamMembership {
    pub fn all(conn: &TeamDBConnection) -> QueryResult<Vec<TeamMembership>> {
        use self::team_memberships::dsl;
        Self::table()
            .filter(dsl::team_public_key.eq(conn.team))
            .get_results(conn.conn)
    }
    pub fn count(conn: &TeamDBConnection) -> Result<u64> {
        use self::team_memberships::dsl;
        Self::table()
            .filter(dsl::team_public_key.eq(conn.team))
            .count()
            .get_result::<i64>(conn.conn)?.to_u64()
    }
    pub fn find(conn: &TeamDBConnection, identity_public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find((conn.team, identity_public_key)).first::<Self>(conn.conn)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn delete(&self, conn: &DBConnection) -> QueryResult<usize> {
        delete(self).execute(conn)
    }
    pub fn update(&self, conn: &DBConnection) -> QueryResult<Self> {
        self.save_changes(conn)
    }
    pub fn filter_admin_public_keys(conn: &TeamDBConnection) -> QueryResult<Vec<Vec<u8>>> {
        use self::team_memberships::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::is_admin.eq(true))
            .get_results::<Self>(conn.conn)
            .map(|admin_memberships| {
                admin_memberships.into_iter().map(|m| m.member_public_key)
                    .collect()
            })
    }

    pub fn filter_admin_emails(conn: &TeamDBConnection) -> QueryResult<Vec<String>> {
        use self::team_memberships::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::is_admin.eq(true))
            .get_results::<Self>(conn.conn)
            .map(|admin_memberships| {
                admin_memberships.into_iter().map(|m| m.email)
                    .collect()
            })
    }

    pub fn find_email(conn: &TeamDBConnection, email: &str) -> QueryResult<Self> {
        use self::team_memberships::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::email.eq(email))
            .first(conn.conn)
    }
    pub fn find_admin(conn: &TeamDBConnection, identity_public_key: &[u8]) -> QueryResult<Self> {
        use self::team_memberships::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::member_public_key.eq(identity_public_key))
            .filter(dsl::is_admin.eq(true))
            .first(conn.conn)
    }

}

#[derive(Queryable, Insertable, Identifiable, Debug, Clone, PartialEq, Eq)]
#[table_name="indirect_invitations"]
#[primary_key(team_public_key, nonce_public_key)]
pub struct IndirectInvitation {
    pub team_public_key: Vec<u8>,
    pub nonce_public_key: Vec<u8>,
    pub restriction_json: String,
    pub invite_symmetric_key_hash: Vec<u8>,
    pub invite_ciphertext: Vec<u8>,
}

impl IndirectInvitation {
    pub fn exists(conn: &TeamDBConnection, nonce_public_key: &[u8]) -> QueryResult<bool> {
        Ok(Self::find(conn, nonce_public_key).optional()?.is_some())
    }
    pub fn find(conn: &TeamDBConnection, nonce_public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find((conn.team, nonce_public_key)).first::<Self>(conn.conn)
    }
    pub fn find_ciphertext(conn: &DBConnection, symmetric_key_hash: &[u8]) -> QueryResult<Self> {
        use self::indirect_invitations::dsl;
        Self::table().filter(dsl::invite_symmetric_key_hash.eq(symmetric_key_hash)).first::<Self>(conn)
    }
    pub fn insert(&self, conn: &TeamDBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn.conn)
    }
    pub fn count(conn: &TeamDBConnection) -> Result<u64> {
        Self::table().filter(indirect_invitations::team_public_key.eq(conn.team))
            .count()
            .get_result::<i64>(conn.conn)?.to_u64()
    }
    pub fn delete_team_invites(conn: &TeamDBConnection) -> QueryResult<()> {
        use self::indirect_invitations::dsl;
        delete(Self::table().filter(dsl::team_public_key.eq(conn.team))).execute(conn.conn)?;
        Ok(())
    }
    pub fn delete(&self, conn: &TeamDBConnection) -> QueryResult<usize> {
        delete(self).execute(conn.conn)
    }
    pub fn to_invitation(self) -> Result<team::IndirectInvitation> {
        Ok(team::IndirectInvitation {
            nonce_public_key: self.nonce_public_key,
            restriction: serde_json::from_str(&self.restriction_json)?,
            invite_symmetric_key_hash: self.invite_symmetric_key_hash,
            invite_ciphertext: self.invite_ciphertext,
        })
    }
    pub fn from_invitation(team_public_key: &[u8], invite: team::IndirectInvitation) -> Result<IndirectInvitation> {
        Ok(IndirectInvitation {
            team_public_key: team_public_key.into(),
            nonce_public_key: invite.nonce_public_key,
            restriction_json: serde_json::to_string(&invite.restriction)?,
            invite_symmetric_key_hash: invite.invite_symmetric_key_hash,
            invite_ciphertext: invite.invite_ciphertext,
        })
    }
}

#[derive(Queryable, Insertable, Identifiable, Debug, Clone, PartialEq, Eq)]
#[table_name="direct_invitations"]
#[primary_key(team_public_key, public_key)]
pub struct DirectInvitation {
    pub team_public_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub email: String,
}

impl DirectInvitation {
    pub fn exists(conn: &TeamDBConnection, public_key: &[u8]) -> QueryResult<bool> {
        Ok(Self::find(conn, public_key).optional()?.is_some())
    }
    pub fn find(conn: &TeamDBConnection, public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find((conn.team, public_key)).first::<Self>(conn.conn)
    }
    pub fn insert(&self, conn: &TeamDBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn.conn)
    }
    pub fn count(conn: &TeamDBConnection) -> Result<u64> {
        Self::table().filter(direct_invitations::team_public_key.eq(conn.team))
            .count()
            .get_result::<i64>(conn.conn)?.to_u64()
    }
    pub fn delete(&self, conn: &TeamDBConnection) -> QueryResult<usize> {
        delete(self).execute(conn.conn)
    }
    pub fn delete_team_invites(conn: &TeamDBConnection) -> QueryResult<()> {
        use self::direct_invitations::dsl;
        delete(Self::table().filter(dsl::team_public_key.eq(conn.team))).execute(conn.conn)?;
        Ok(())
    }
    pub fn to_invitation(self) -> team::DirectInvitation {
        team::DirectInvitation {
            public_key: self.public_key,
            email: self.email,
        }
    }
    pub fn from_invitation(team_public_key: &[u8], invite: team::DirectInvitation) -> DirectInvitation {
        DirectInvitation {
            team_public_key: team_public_key.into(),
            public_key: invite.public_key,
            email: invite.email,
        }
    }
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Debug, Clone)]
#[table_name="identities"]
#[primary_key(team_public_key, public_key)]
pub struct Identity {
    pub team_public_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub encryption_public_key: Vec<u8>,
    pub ssh_public_key: Vec<u8>,
    pub pgp_public_key: Vec<u8>,
    pub email: String,
}

impl Identity {
    pub fn from_identity(team: Vec<u8>, i: team::Identity) -> Identity {
        Identity {
            team_public_key: team,
            public_key: i.public_key,
            encryption_public_key: i.encryption_public_key,
            ssh_public_key: i.ssh_public_key,
            pgp_public_key: i.pgp_public_key,
            email: i.email,
        }
    }
    pub fn into_identity(self) -> team::Identity {
        team::Identity{
            public_key: self.public_key,
            encryption_public_key: self.encryption_public_key,
            ssh_public_key: self.ssh_public_key,
            pgp_public_key: self.pgp_public_key,
            email: self.email,
        }
    }
    pub fn find(conn: &TeamDBConnection, identity_public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find((conn.team, identity_public_key)).first::<Self>(conn.conn)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn insert_or_update(&self, conn: &DBConnection) -> Result<()> {
        if let Some(_existing) = Self::table().find(self.id()).first::<Self>(conn).optional()? {
            self.save_changes::<Self>(conn)?;
        } else {
            insert_into(Self::table()).values(self).execute(conn)?;
        }
        Ok(())
    }
    pub fn delete(&self, conn: &DBConnection) -> QueryResult<usize> {
        delete(self).execute(conn)
    }
    pub fn find_all_for_team(conn: &TeamDBConnection) -> QueryResult<Vec<Self>> {
        use self::identities::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
                     .get_results(conn.conn)
    }
    pub fn filter_by_email(conn: &DBConnection, team_public_key: &[u8], email: &str) -> QueryResult<Vec<Self>> {
        use self::identities::dsl;
        Self::table().filter(dsl::team_public_key.eq(team_public_key))
            .filter(dsl::email.eq(email))
            .get_results(conn)
    }
    pub fn filter_by_encryption_public_key(conn: &TeamDBConnection, encryption_public_key: &[u8]) -> QueryResult<Vec<Self>> {
        use self::identities::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::encryption_public_key.eq(encryption_public_key))
            .get_results(conn.conn)
    }
    pub fn filter_by_public_keys(conn: &TeamDBConnection, identity_public_keys: &Vec<Vec<u8>>) -> QueryResult<Vec<Self>> {
        use self::identities::dsl;
        Self::table().filter(dsl::team_public_key.eq(conn.team))
            .filter(dsl::public_key.eq_any(identity_public_keys))
            .get_results(conn.conn)
    }
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset, Debug, Clone)]
#[table_name="teams"]
#[primary_key(public_key)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Team {
    pub public_key: Vec<u8>,
    pub last_block_hash: Vec<u8>,
    pub name: String,
    pub temporary_approval_seconds: Option<i64>,

    // Client only
    pub last_read_log_chain_logical_timestamp: Option<i64>,

    pub command_encrypted_logging_enabled: bool,
}

impl Team {
    pub fn find(conn: &TeamDBConnection) -> QueryResult<Self> {
        Self::table().find(conn.team).first::<Self>(conn.conn)
    }
    pub fn created_at(conn: &TeamDBConnection) -> QueryResult<chrono::NaiveDateTime> {
        Ok(Block::table().filter(blocks::team_public_key.eq(conn.team))
            .filter(blocks::last_block_hash.is_null())
            .first::<Block>(conn.conn)?.created_at.clone())
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn update_last_block_hash(conn: &TeamDBConnection, last_block_hash: &[u8]) -> QueryResult<usize> {
        use db::teams::dsl;
        update(Self::table().find(conn.team))
            .set(dsl::last_block_hash.eq(last_block_hash))
            .execute(conn.conn)
    }
    pub fn update(&self, conn: &DBConnection) -> QueryResult<Self> {
        self.save_changes(conn)
    }
}

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
#[table_name="pinned_host_keys"]
#[primary_key(team_public_key, host, public_key)]
pub struct PinnedHostKey {
    pub team_public_key: Vec<u8>,
    pub host: String,
    pub public_key: Vec<u8>,
}

impl PinnedHostKey {
    pub fn exists(&self, conn: &DBConnection) -> QueryResult<bool> {
        select(exists(Self::table().find(self.id()))).get_result(conn)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn delete(&self, conn: &DBConnection) -> QueryResult<usize> {
        delete(self).execute(conn)
    }
    pub fn filter_by_host(
        conn: &DBConnection,
        team_public_key: &[u8],
        host: &str,
        search: bool) -> QueryResult<Vec<Self>> {
        use self::pinned_host_keys::dsl;
        let team_filter = Self::table().filter(dsl::team_public_key.eq(team_public_key));
        if search {
            team_filter.filter(dsl::host.like(format!("%{}%", host)))
                .get_results(conn)
        } else {
            team_filter.filter(dsl::host.eq(host))
                       .get_results(conn)
        }
    }
    pub fn find_all_for_team(conn: &DBConnection, team_public_key: &[u8]) -> QueryResult<Vec<Self>> {
        use self::pinned_host_keys::dsl;
        Self::table().filter(dsl::team_public_key.eq(team_public_key)).get_results(conn)
    }
    pub fn count(conn: &TeamDBConnection) -> Result<u64> {
        use self::pinned_host_keys::dsl;
        Self::table()
            .filter(dsl::team_public_key.eq(conn.team))
            .count()
            .get_result::<i64>(conn.conn)?
            .to_u64()
    }
}

impl Into<team::SSHHostKey> for PinnedHostKey {
    fn into(self) -> team::SSHHostKey {
        team::SSHHostKey {
            host: self.host,
            public_key: self.public_key,
        }
    }
}

#[derive(Queryable, Insertable, Identifiable, AsChangeset, Debug, Clone, PartialEq, Eq)]
#[table_name="log_chains"]
#[primary_key(team_public_key, member_public_key)]
#[changeset_options(treat_none_as_null = "true")]
pub struct LogChain {
    pub team_public_key: Vec<u8>,
    pub member_public_key: Vec<u8>,
    pub last_block_hash: Vec<u8>,
    pub symmetric_encryption_key: Option<Vec<u8>>,
}

impl LogChain {
    pub fn find(conn: &TeamDBConnection, member_public_key: &[u8]) -> QueryResult<Self> {
        Self::table().find((conn.team, member_public_key)).first::<Self>(conn.conn)
    }
    pub fn insert(&self, conn: &DBConnection) -> QueryResult<usize> {
        insert_into(Self::table()).values(self).execute(conn)
    }
    pub fn update_last_block_hash(
        conn: &TeamDBConnection,
        member_public_key: &[u8],
        last_block_hash: Vec<u8>,
    ) -> QueryResult<usize> {
        use self::log_chains::dsl;
        update(Self::table().find((conn.team, member_public_key)))
            .set(dsl::last_block_hash.eq(last_block_hash))
            .execute(conn.conn)
    }
    pub fn update_symmetric_encryption_key(
        conn: &TeamDBConnection,
        member_public_key: &[u8],
        symmetric_encrpytion_key: Option<Vec<u8>>,
    ) -> QueryResult<usize> {
        use self::log_chains::dsl;
        update(Self::table().find((conn.team, member_public_key)))
            .set(dsl::symmetric_encryption_key.eq(symmetric_encrpytion_key))
            .execute(conn.conn)
    }
    pub fn update(&self, conn: &DBConnection) -> QueryResult<Self> {
        self.save_changes(conn)
    }
}
