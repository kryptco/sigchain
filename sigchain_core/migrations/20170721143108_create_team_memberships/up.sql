CREATE TABLE team_memberships (
	team_public_key BYTEA,
	member_public_key BYTEA,
	email VARCHAR NOT NULL,
	is_admin BOOLEAN NOT NULL,
	PRIMARY KEY (team_public_key, member_public_key)
);
CREATE UNIQUE INDEX team_memberships_team_public_key_email on team_memberships (team_public_key, email)
