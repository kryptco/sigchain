CREATE TABLE indirect_invitations (
	team_public_key BYTEA,
	nonce_public_key BYTEA NOT NULL,
	restriction_json VARCHAR NOT NULL,
	invite_symmetric_key_hash BYTEA NOT NULL,
	invite_ciphertext BYTEA NOT NULL,
	PRIMARY KEY (team_public_key, nonce_public_key)
);
CREATE UNIQUE INDEX indirect_invitations_invite_symmetric_key_hash_index ON indirect_invitations (invite_symmetric_key_hash)
