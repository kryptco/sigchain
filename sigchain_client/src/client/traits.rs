use super::*;
use itertools;

pub trait Broadcast {
    fn broadcast<'a, T>(&self, endpoint: &protocol::Endpoint, request: &SignedMessage) -> super::Result<T>
        where T: serde::de::DeserializeOwned;
}

pub trait DBConnect: Identify {
    fn db_conn(&self) -> &super::DBConnection;
    fn team_db_conn(&self) -> db::TeamDBConnection {
        db::TeamDBConnection{team: self.team_pk(), conn: self.db_conn()}
    }

    fn get_last_block_hash(&self) -> Result<Option<Vec<u8>>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(db::Team::find(conn).optional()?.map(|t| t.last_block_hash))
    }
    fn team_pointer(&self) -> Result<TeamPointer> {
        Ok(match self.get_last_block_hash()? {
            None => TeamPointer::PublicKey(self.team_pk().into()),
            Some(last_block_hash) => TeamPointer::LastBlockHash(last_block_hash),
        })
    }
    fn my_log_pointer(&self) -> Result<LogChainPointer> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: &self.team_pk()};
        if let Some(log_chain) = db::LogChain::find(conn, self.identity_pk()).optional()? {
            Ok(LogChainPointer::LastBlockHash(log_chain.last_block_hash))
        } else {
            Ok(
                LogChainPointer::GenesisBlock(LogChainGenesisPointer{
                    team_public_key: self.team_pk().into(),
                    member_public_key: self.identity_pk().into(),
                })
            )
        }
    }
    fn get_admins(&self) -> Result<Vec<team::Identity>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let admin_public_keys = db::TeamMembership::filter_admin_public_keys(conn)?;
        Ok(
            db::Identity::filter_by_public_keys(conn, &admin_public_keys)?.into_iter()
                .map(db::Identity::into_identity).collect()
        )
    }
    fn get_admin_by_email(&self, email: &str) -> Result<team::Identity> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let admin_pk = db::TeamMembership::find_admin_by_email(conn, email)?.member_public_key;
        Ok(db::Identity::find(conn, &admin_pk).map(db::Identity::into_identity)?)
    }
    fn get_active_and_removed_members(&self) -> Result<Vec<team::Identity>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let results = db::Identity::find_all_for_team(conn)?;
        Ok(results.into_iter().map(db::Identity::into_identity).collect())
    }

    fn get_active_member_by_email(&self, email: &str) -> Result<team::Identity> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        match db::TeamMembership::find_email(conn, email).optional()? {
            Some(member) => Ok(
                db::Identity::find(
                    conn,
                    &member.member_public_key,
                )?.into_identity()
            ),
            None => bail!(format!("no active member found for email: {:?}", email)),
        }
    }

    fn get_active_and_removed_by_email(&self, email: &str) -> Result<Vec<team::Identity>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(
            db::Identity::filter_by_email(conn, email)?.into_iter()
                .map(db::Identity::into_identity).collect::<Vec<_>>()
        )
    }

    fn get_active_members(&self) -> Result<Vec<team::Identity>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let results = db::Identity::find_all_for_team(conn)?;

        // Filter all members and their membership status.
        let members = itertools::process_results(
            results.into_iter().map(|m| -> Result<Option<Identity>> {
                Ok(
                    // Only take the active members.
                    db::TeamMembership::find(conn, &m.public_key).optional()?
                        .map(|_| m.into_identity())
                )
            }),
            |i| i.filter_map(|i| i).collect::<Vec<_>>()
        )?;

        Ok(members)
    }

    fn get_my_identity(&self) -> Result<team::Identity> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(db::Identity::find(conn, self.identity_pk()).map(db::Identity::into_identity)?)
    }
    fn is_admin(&self) -> Result<bool> {
        let conn = &db::TeamDBConnection { conn: self.db_conn(), team: self.team_pk() };
        Ok(db::TeamMembership::find(conn, self.identity_pk())?.is_admin)
    }
    fn get_team_info(&self) -> Result<team::TeamInfo> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let name = db::Team::find(conn)?.name;
        Ok(TeamInfo{name})
    }
    fn is_command_encrypted_logging_enabled(&self) -> Result<bool> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(db::Team::find(conn)?.command_encrypted_logging_enabled)
    }
    fn get_policy(&self) -> Result<team::Policy> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        let temporary_approval_seconds = db::Team::find(conn)?.temporary_approval_seconds;
        Ok(Policy{
            temporary_approval_seconds,
        })
    }
    fn get_pinned_host_keys(&self, for_host: &str, search: bool) -> Result<Vec<db::PinnedHostKey>> {
        let conn = self.db_conn();
        let host_keys =
            db::PinnedHostKey::filter_by_host(conn, &self.team_pk(), for_host, search)?;
        Ok(host_keys)
    }
    fn get_all_pinned_host_keys(&self) -> Result<Vec<SSHHostKey>> {
        let conn = self.db_conn();
        Ok(
            db::PinnedHostKey::find_all_for_team(conn, &self.team_pk())?
                .into_iter().map(db::PinnedHostKey::into).collect()
        )
    }
    fn get_encryption_public_key(&self, identity_public_key: &[u8]) -> Result<Vec<u8>> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(db::Identity::find(conn, identity_public_key)?.encryption_public_key)
    }
    fn main_chain_block_count(&self) -> Result<u64> {
        let conn = &db::TeamDBConnection{conn: self.db_conn(), team: self.team_pk()};
        Ok(db::Block::count(conn)?)
    }
}

pub trait Identify {
    fn identity_pk(&self) -> &[u8];
    fn team_pk(&self) -> &[u8];
}
