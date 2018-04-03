table! {
    current_team (team_checkpoint) {
        team_checkpoint -> Binary,
        sign_key_pair -> Nullable<Binary>,
        box_key_pair -> Nullable<Binary>,
    }
}

table! {
    logs (id) {
        id -> BigInt,
        team_public_key -> Binary,
        member_public_key -> Binary,
        log_json -> Text,
        unix_seconds -> BigInt,
    }
}

table! {
    read_tokens (team_public_key) {
        team_public_key -> Binary,
        token -> Binary,
        reader_key_pair -> Binary,
    }
}

table! {
    current_wrapped_keys (destination_public_key) {
        destination_public_key -> Binary,
    }
}

table! {
    queued_logs (id) {
        id -> BigInt,
        log_json -> Binary,
    }
}
