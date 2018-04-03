CREATE TABLE pinned_host_keys (
	team_public_key BYTEA,
	host VARCHAR,
	public_key BYTEA,
	PRIMARY KEY (team_public_key, host, public_key)
)
