CREATE TABLE log_blocks (
	hash BYTEA PRIMARY KEY,
	last_block_hash BYTEA,
	team_public_key BYTEA NOT NULL,
	member_public_key BYTEA NOT NULL,
	operation VARCHAR NOT NULL,
	signature BYTEA NOT NULL,
	created_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX log_blocks_team_public_key_member_public_key_last_block_hash_index ON log_blocks (team_public_key, member_public_key, last_block_hash);

CREATE UNIQUE INDEX log_blocks_team_public_key_hash_index ON log_blocks (team_public_key, hash);
CREATE UNIQUE INDEX log_blocks_member_public_key_hash_index ON log_blocks (member_public_key, hash);

CREATE INDEX log_blocks_team_public_key_created_at_index ON log_blocks (team_public_key, created_at)
