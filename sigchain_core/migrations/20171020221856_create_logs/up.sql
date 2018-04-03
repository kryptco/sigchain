CREATE TABLE logs (
    id INTEGER PRIMARY KEY,
    team_public_key BYTEA,
    member_public_key BYTEA,
    log_json VARCHAR,
    unix_seconds BIGINT NOT NULL
);
CREATE INDEX logs_team_public_key_unix_seconds_index ON logs (team_public_key, unix_seconds);
CREATE INDEX logs_team_public_key_id_index ON logs (team_public_key, id)
