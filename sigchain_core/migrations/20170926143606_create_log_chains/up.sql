CREATE TABLE log_chains (
	team_public_key BYTEA,
	member_public_key BYTEA,
	last_block_hash BYTEA NOT NULL,
	symmetric_encryption_key BYTEA,
	PRIMARY KEY (team_public_key, member_public_key)
)
