CREATE TABLE direct_invitations (
	team_public_key BYTEA,
	public_key BYTEA NOT NULL,
	email VARCHAR NOT NULL,
	PRIMARY KEY (team_public_key, public_key)
);
CREATE UNIQUE INDEX direct_invitations_team_public_key_email_index on direct_invitations (team_public_key, email)
