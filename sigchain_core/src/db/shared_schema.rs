table! {
    identities (team_public_key, public_key) {
        team_public_key -> Binary,
        public_key -> Binary,
        encryption_public_key -> Binary,
        ssh_public_key -> Binary,
        pgp_public_key -> Binary,
        email -> Text,
    }
}

table! {
    blocks (hash) {
        hash -> Binary,
        last_block_hash -> Nullable<Binary>,
        team_public_key -> Binary,
        member_public_key -> Binary,
        operation -> Text,
        signature -> Binary,
        created_at -> Timestamp,
    }
}

table! {
    log_blocks (hash) {
        hash -> Binary,
        last_block_hash -> Nullable<Binary>,
        team_public_key -> Binary,
        member_public_key -> Binary,
        operation -> Text,
        signature -> Binary,
        created_at -> Timestamp,
    }
}

table! {
    teams (public_key) {
        public_key -> Binary,
        last_block_hash -> Binary,
        name -> Text,
        temporary_approval_seconds -> Nullable<BigInt>,
        last_read_log_chain_logical_timestamp -> Nullable<BigInt>,
        command_encrypted_logging_enabled -> Bool,
    }
}

table! {
    team_memberships (team_public_key, member_public_key) {
        team_public_key -> Binary,
        member_public_key -> Binary,
        email -> Text,
        is_admin -> Bool,
    }
}

table! {
    indirect_invitations (team_public_key, nonce_public_key) {
        team_public_key -> Binary,
        nonce_public_key -> Binary,
        restriction_json -> Text,
        invite_symmetric_key_hash -> Binary,
        invite_ciphertext -> Binary,
    }
}

table! {
    direct_invitations (team_public_key, public_key) {
        team_public_key -> Binary,
        public_key -> Binary,
        email -> Text,
    }
}

table! {
    pinned_host_keys (team_public_key, host, public_key) {
        team_public_key -> Binary,
        host -> Text,
        public_key -> Binary,
    }
}

table! {
    log_chains (team_public_key, member_public_key) {
        team_public_key -> Binary,
        member_public_key -> Binary,
        last_block_hash -> Binary,
        symmetric_encryption_key -> Nullable<Binary>,
    }
}
