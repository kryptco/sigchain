CREATE TABLE identities (
	team_public_key BYTEA,
	public_key BYTEA,
	encryption_public_key BYTEA NOT NULL,
	ssh_public_key BYTEA NOT NULL,
	pgp_public_key BYTEA NOT NULL,
	email VARCHAR NOT NULL,
	PRIMARY KEY (team_public_key, public_key)
);
CREATE INDEX identities_team_public_key_email_index ON identities (team_public_key, email);
CREATE INDEX identities_team_public_key_encryption_public_key_index ON identities (team_public_key, encryption_public_key) -- for clients to look up sender encryption_public_keys
