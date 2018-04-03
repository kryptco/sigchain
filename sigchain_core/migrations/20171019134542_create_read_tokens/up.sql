CREATE TABLE read_tokens (
	team_public_key BYTEA PRIMARY KEY,
	token BYTEA NOT NULL,
	reader_key_pair BYTEA NOT NULL
)
