use super::*;

gen_test!(admin_dir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    // Try to invite new admin to team.
    let admin_dir_invite_block = dir_invite_user_block(
        &users[0], &user, &user_promote_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
    blocks.push(admin_dir_invite_block);
});

gen_test!(admin_dir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Try to invite self to team.
    let self_dir_invite_block = dir_invite_user_block(
        &users[0], &users[0], &blocks.last().unwrap().hash(), false);

    blocks.push(self_dir_invite_block);
});

gen_test!(admin_dir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Try to invite user.
    let user_dir_invite_block = dir_invite_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_dir_invite_block);
});

gen_test!(admin_dir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (direct) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let dir_user_invite_block = dir_invite_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), true);

    users.push(user);
    blocks.push(dir_user_invite_block);
});

gen_test!(admin_dir_invite_non_member_accept,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    users.push(user);
    blocks.extend(user_add_blocks);
});

gen_test!(dir_invite_remove_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User is removed and then tries to reaccept invite.
    let user_remove_block = remove_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_dir_invite_block(&user, &user_remove_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_remove_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(dir_invite_leave_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User leaves team and then tries to reaccept invite.
    let user_leave_block = leave_team_block(&user, &user_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_dir_invite_block(&user, &user_leave_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_leave_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(dir_invite_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to reaccept invite.
    let reaccept_invite_block = accept_dir_invite_block(
        &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(reaccept_invite_block);
});

gen_test!(closed_dir_invite,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (direct) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_dir_invite_block = dir_invite_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), true);

    // Close all invites.
    let close_invite_block = close_invites_block(&users[0], &user_dir_invite_block.hash(), true);

    // User tries to accept closed invite.
    let accept_invite_block = accept_dir_invite_block(&user, &close_invite_block.hash(), false);

    users.push(user);
    blocks.push(user_dir_invite_block);
    blocks.push(close_invite_block);
    blocks.push(accept_invite_block);
});

gen_test!(member_dir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to direct invite admin.
    let admin_invite_block = dir_invite_user_block(
        &user, &users[0], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_invite_block);
});

gen_test!(member_dir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate and add (direct invite and accept) user 2 to team.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    // User 1 tries to direct invite user 2.
    let user_2_invite_block = dir_invite_user_block(
        &user_1, &user_2, &user_2_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
    blocks.push(user_2_invite_block);
});

gen_test!(member_dir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to direct invite self.
    let self_invite_block = dir_invite_user_block(
        &user, &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(self_invite_block);
});

gen_test!(member_dir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to direct invite user 2.
    let user_2_invite_block = dir_invite_user_block(
        &user_1, &user_2, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_2_invite_block);
});

gen_test!(non_member_dir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to direct invite admin.
    let admin_invite_block = dir_invite_user_block(
        &user, &users[0], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(admin_invite_block);
});

gen_test!(non_member_dir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 2 tries to direct invite user 1.
    let user_1_invite_block = dir_invite_user_block(
        &user_2, &user_1, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_1_invite_block);
});

gen_test!(non_member_dir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to direct invite user 2.
    let user_2_invite_block = dir_invite_user_block(
        &user_1, &user_2, &blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.push(user_2_invite_block);
});

gen_test!(non_member_dir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to direct invite self.
    let user_invite_block = dir_invite_user_block(
        &user, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_invite_block);
});

gen_test!(dir_invite_user_hijack,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2 and 3.
    let mut user_2 = generate_user(&users[0].client.team_public_key, 2);
    user_2.client.sign_key_pair_seed = user_1.client.sign_key_pair_seed.clone();
    user_2.sign_key_pair = user_1.sign_key_pair.clone();
    let mut user_3 = generate_user(&users[0].client.team_public_key, 3);
    user_3.email = user_1.email.clone();

    // Invite (direct) user 1 to team.
    let user_1_invite_block = dir_invite_user_block(
        &users[0], &user_1, &blocks.last().unwrap().hash(), true);

    // Try to get user 2 to hijack invite.
    let user_2_accept_invite_block = accept_dir_invite_block(
        &user_2, &user_1_invite_block.hash(), false);

    // Try to get user 3 to hijack invite.
    let user_3_accept_invite_block = accept_dir_invite_block(
        &user_3, &user_1_invite_block.hash(), false);

    users.push(user_1);
    users.push(user_2);
    users.push(user_3);
    blocks.push(user_1_invite_block);
    blocks.push(user_2_accept_invite_block);
    blocks.push(user_3_accept_invite_block);
});

gen_test!(dir_invite_team_hijack,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set up another creator and team.
    let (creator_2, team_creation_block_2) = setup_team_block(None);

    // Generate user.
    let user = generate_user(&creator_2.client.team_public_key, 1);

    // Admin 1 tries to invite (direct) user to team 2.
    let mut user_invite_block = dir_invite_user_block(
        &users[0], &user, &team_creation_block_2.hash(), false);
    user_invite_block.expected.team_public_key = creator_2.sign_key_pair.public_key_bytes().into();

    users.push(creator_2);
    users.push(user);
    blocks.push(team_creation_block_2);
    blocks.push(user_invite_block);
});

gen_test!(dir_invite_accept_id_sig_mismatch_key,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Invite (direct) user to team.
    let user_invite_block = dir_invite_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), true);

    let user_accept_inv_msg = SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: user_invite_block.hash(),
                operation: AcceptInvite(generate_identity(
                    &gen_sign_key_pair().unwrap(),
                    &user.client.box_key_pair,
                    &user.email,
                )),
            })),
        },
        &user.sign_key_pair,
    ).unwrap();

    let user_accept_inv_block = TestBlock {
        signed_message: user_accept_inv_msg,
        expected: ExpectedResult {
            valid: false,
            team_public_key: user_invite_block.expected.team_public_key.clone(),
        },
    };

    users.push(user);
    blocks.push(user_invite_block);
    blocks.push(user_accept_inv_block);
});

gen_test!(dir_invite_accept_pk_sig_mismatch_key,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Generate and add (direct invite and accept) user to team.
    let mut user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Switch out the public key of the accept invite block with the admin's.
    user_add_blocks.last_mut().unwrap().signed_message.public_key = users[0].sign_key_pair
        .public_key_bytes()
        .into();
    user_add_blocks.last_mut().unwrap().expected.valid = false;

    users.push(user);
    blocks.extend(user_add_blocks);
});

gen_test!(dir_invite_duplicate,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Invite (direct) user to team.
    let user_invite_block = dir_invite_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), true);

    // Try to invite (direct) user to team again.
    let user_invite_again_block = dir_invite_user_block(
        &users[0], &user, &user_invite_block.hash(), false);

    users.push(user);
    blocks.push(user_invite_block);
    blocks.push(user_invite_again_block);
});
