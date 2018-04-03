use super::*;

gen_test!(create_team,
|_users: &mut Vec<User>, _blocks: &mut Vec<TestBlock>| {});

gen_test!(semver_reject,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Try to write a block with larger major protocol version than what clients support.
    let mut next_version = CURRENT_VERSION.clone();
    next_version.increment_major();
    let admin_add_logging_block = add_logging_with_version_block(
        next_version, &users[0], &blocks.last().unwrap().hash(), false);

    blocks.push(admin_add_logging_block);
});

gen_test!(consume_dir_invite,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Directly invite user to team.
    let user_dir_invite_block = dir_invite_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), true);

    // Indirectly invite user to team and have them accept.
    let (temp_user, user_add_blocks) = add_user_indir_blocks(
        &users[0], &user, &user_dir_invite_block.hash());

    // User leaves the team.
    let user_leave_block = leave_team_block(&user, &user_add_blocks.last().unwrap().hash(), true);

    // User tries to accept direct invite.
    let user_accept_dir_invite_block = accept_dir_invite_block(
        &user, &user_leave_block.hash(), false);

    users.push(user);
    users.push(temp_user);
    blocks.push(user_dir_invite_block);
    blocks.extend(user_add_blocks);
    blocks.push(user_leave_block);
    blocks.push(user_accept_dir_invite_block);
});

gen_test!(create_team_id_sig_mismatch_key,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    let team_creation_msg = SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Create(GenesisBlock {
                team_info: TeamInfo {
                    name: String::from("Acme Engineering"),
                },
                creator_identity: generate_identity(
                    &gen_sign_key_pair().unwrap(),
                    &users[0].client.box_key_pair,
                    &users[0].email,
                ),
            }))
        },
        &users[0].sign_key_pair,
    ).unwrap();

    let team_creation_block = TestBlock {
        signed_message: team_creation_msg,
        expected: ExpectedResult {
            valid: false,
            team_public_key: users[0].client.team_public_key.clone(),
        },
    };

    blocks[0] = team_creation_block;
});

gen_test!(create_team_pk_sig_mismatch_key,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user to use their public key.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Switch out the public key of the create team block with the user's.
    blocks.last_mut().unwrap().signed_message.public_key = user.sign_key_pair
        .public_key_bytes()
        .into();
    blocks.last_mut().unwrap().expected.valid = false;

    users.push(user);
});

gen_test!(admin_set_policy,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set policy.
    let admin_set_policy_block = set_policy_block(
        10800, &users[0], &blocks.last().unwrap().hash(), true);

    blocks.push(admin_set_policy_block);
});

gen_test!(member_set_policy,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let add_user_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to set policy.
    let user_set_policy_block = set_policy_block(
        10800, &user, &add_user_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(add_user_blocks);
    blocks.push(user_set_policy_block);
});

gen_test!(non_member_set_policy,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to set policy.
    let user_set_policy_block = set_policy_block(
        10800, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_set_policy_block);
});

gen_test!(duplicate_set_policy,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set same policy twice.
    let admin_set_policy_block = set_policy_block(
        10800, &users[0], &blocks.last().unwrap().hash(), true);
    let admin_set_policy_again_block = set_policy_block(
        10800, &users[0], &admin_set_policy_block.hash(), true);

    blocks.push(admin_set_policy_block);
    blocks.push(admin_set_policy_again_block);
});

gen_test!(admin_set_team_info,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set team info.
    let admin_set_team_info_block = set_team_info_block(
        "Test", &users[0], &blocks.last().unwrap().hash(), true);

    blocks.push(admin_set_team_info_block);
});

gen_test!(member_set_team_info,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to set team info.
    let user_set_team_info_block = set_team_info_block(
        "Test", &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_set_team_info_block);
});

gen_test!(non_member_set_team_info,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to set team info.
    let user_set_team_info_block = set_team_info_block(
        "Test", &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_set_team_info_block);
});

gen_test!(duplicate_set_team_info,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set same team info twice.
    let admin_set_team_info_block = set_team_info_block(
        "Test", &users[0], &blocks.last().unwrap().hash(), true);
    let admin_set_team_info_again_block = set_team_info_block(
        "Test", &users[0], &admin_set_team_info_block.hash(), true);

    blocks.push(admin_set_team_info_block);
    blocks.push(admin_set_team_info_again_block);
});

gen_test!(duplicate_encryption_public_key_on_team,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add two users to team with the same encryption public key.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let mut user_2 = generate_user(&users[0].client.team_public_key, 2);
    user_2.client.box_key_pair = user_1.client.box_key_pair.clone();

    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
});
