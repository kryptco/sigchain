CREATE TABLE teams (
	public_key BYTEA PRIMARY KEY,
	last_block_hash BYTEA,
	name VARCHAR NOT NULL,
	temporary_approval_seconds BIGINT,
	last_read_log_chain_logical_timestamp BIGINT, -- client only
	command_encrypted_logging_enabled BOOLEAN NOT NULL
)
