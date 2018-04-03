use super::*;

gen_test!(admin_indir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    // Try to invite new admin to team.
    let (_nonce_key_pair_seed, admin_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &user_promote_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
    blocks.push(admin_indir_invite_block);
});

gen_test!(admin_indir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Try to invite self to team.
    let(_nonce_key_pair_seed, self_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&users[0]], &blocks.last().unwrap().hash(), false);

    blocks.push(self_indir_invite_block);
});

gen_test!(admin_indir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Try to invite user.
    let (_nonce_key_pair_seed, user_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_indir_invite_block);
});

gen_test!(admin_indir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (indirect) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (_nonce_key_pair_seed, user_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &blocks.last().unwrap().hash(), true);

    users.push(user);
    blocks.push(user_indir_invite_block);
});

gen_test!(admin_indir_invite_non_member_accept,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, user_add_blocks) = add_user_indir_blocks(
        &users[0], &user, &blocks.last().unwrap().hash());

    users.push(user);
    users.push(temp_user);
    blocks.extend(user_add_blocks);
});

gen_test!(admin_indir_invite_many_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (indirect) two users to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let (_nonce_key_pair_seed, users_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user_1, &user_2], &blocks.last().unwrap().hash(), true);

    users.push(user_1);
    users.push(user_2);
    blocks.push(users_indir_invite_block);
});

gen_test!(admin_indir_invite_mixed,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate two users.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // Add one to team and then try to indirect invite both.
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());
    let (_nonce_key_pair_seed, users_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user_1, &user_2], &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(users_indir_invite_block);
});

gen_test!(indir_invite_remove_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, user_add_blocks) = add_user_indir_blocks(
        &users[0], &user, &blocks.last().unwrap().hash());

    // User is removed and then tries to reaccept invite.
    let user_remove_block = remove_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed, &user, &user_remove_block.hash(), false);

    users.push(user);
    users.push(temp_user);
    blocks.extend(user_add_blocks);
    blocks.push(user_remove_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(indir_invite_leave_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, user_add_blocks) = add_user_indir_blocks(
        &users[0], &user, &blocks.last().unwrap().hash());

    // User leaves and then tries to reaccept invite.
    let user_leave_block = leave_team_block(&user, &user_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed, &user, &user_leave_block.hash(), true);

    users.push(user);
    users.push(temp_user);
    blocks.extend(user_add_blocks);
    blocks.push(user_leave_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(indir_invite_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, user_add_blocks) = add_user_indir_blocks(
        &users[0], &user, &blocks.last().unwrap().hash());

    // User tries to reaccept invite.
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed,
        &user,
        &user_add_blocks.last().unwrap().hash(),
        false,
    );

    users.push(user);
    users.push(temp_user);
    blocks.extend(user_add_blocks);
    blocks.push(reaccept_invite_block);
});

gen_test!(closed_indir_invite,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (indirect) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (nonce_key_pair_seed, user_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &blocks.last().unwrap().hash(), true);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: users[0].client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    // Close all invites.
    let close_invite_block = close_invites_block(&users[0], &user_indir_invite_block.hash(), true);

    // User tries to accept closed invite.
    let user_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user, &close_invite_block.hash(), false);

    users.push(user);
    users.push(temp_user);
    blocks.push(user_indir_invite_block);
    blocks.push(close_invite_block);
    blocks.push(user_accept_invite_block);
});

gen_test!(member_indir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to indirect invite admin.
    let(_nonce_key_pair_seed, admin_invite_block) = indir_invite_users_block(
        &user, &[&users[0]], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_invite_block);
});

gen_test!(member_indir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate and add (direct invite and accept) user 2 to team.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    // User 1 tries to indirect invite user 2.
    let (_nonce_key_pair_seed, user_2_invite_block) = indir_invite_users_block(
        &user_1, &[&user_2], &user_2_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
    blocks.push(user_2_invite_block);
});

gen_test!(member_indir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to indirect invite self.
    let (_nonce_key_pair_seed, user_invite_block) = indir_invite_users_block(
        &user, &[&user], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_invite_block);
});

gen_test!(member_indir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to indirect invite user 2.
    let (_nonce_key_pair_seed, user_2_invite_block) = indir_invite_users_block(
        &user_1, &[&user_2], &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_2_invite_block);
});

gen_test!(non_member_indir_invite_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to indirect invite admin.
    let (_nonce_key_pair_seed, admin_invite_block) = indir_invite_users_block(
        &user, &[&users[0]], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(admin_invite_block);
});

gen_test!(non_member_indir_invite_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 2 tries to indirect invite user 1.
    let (_nonce_key_pair_seed, user_1_invite_block) = indir_invite_users_block(
        &user_2, &[&user_2], &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_1_invite_block);
});

gen_test!(non_member_indir_invite_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to indirect invite user 2.
    let (_nonce_key_pair_seed, user_2_invite_block) = indir_invite_users_block(
        &user_1, &[&user_2], &blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.push(user_2_invite_block);
});

gen_test!(non_member_indir_invite_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to indirect invite self.
    let (_nonce_key_pair_seed, user_invite_block) = indir_invite_users_block(
        &user, &[&user], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_invite_block);
});

gen_test!(indir_invite_user_hijack,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2 and 3.
    let mut user_2 = generate_user(&users[0].client.team_public_key, 2);
    user_2.client.sign_key_pair_seed = user_1.client.sign_key_pair_seed.clone();
    user_2.sign_key_pair = user_1.sign_key_pair.clone();
    let mut user_3 = generate_user(&users[0].client.team_public_key, 3);
    user_3.email = user_1.email.clone();

    // Invite (indirect) user 1 to team.
    let (nonce_key_pair_seed, user_1_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user_1], &blocks.last().unwrap().hash(), true);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: users[0].client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    // Try to get user 2 to hijack invite.
    let user_2_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user_2, &user_1_indir_invite_block.hash(), false);

    // User 3 should be able to accept invite.
    let user_3_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user_3, &user_1_indir_invite_block.hash(), true);

    users.push(user_1);
    users.push(user_2);
    users.push(user_3);
    users.push(temp_user);
    blocks.push(user_1_indir_invite_block);
    blocks.push(user_2_accept_invite_block);
    blocks.push(user_3_accept_invite_block);
});

gen_test!(indir_invite_team_hijack,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Set up another creator and team.
    let (creator_2, team_creation_block_2) = setup_team_block(None);

    // Generate user.
    let user = generate_user(&creator_2.client.team_public_key, 1);

    // Admin 1 tries to invite (indirect) user to team 2.
    let (_nonce_key_pair_seed, mut user_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &team_creation_block_2.hash(), false);
    user_invite_block.expected.team_public_key = creator_2.sign_key_pair.public_key_bytes().into();

    users.push(creator_2);
    users.push(user);
    blocks.push(team_creation_block_2);
    blocks.push(user_invite_block);
});

gen_test!(admin_indir_invite_domain,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate non-member, member, and admin.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(&users[0], &user_2, &blocks.last().unwrap().hash());
    let user_3 = generate_user(&users[0].client.team_public_key, 3);
    let user_3_add_blocks = add_user_blocks(
        &users[0], &user_3, &user_2_add_blocks.last().unwrap().hash());
    let user_3_promote_block = promote_user_block(
        &users[0], &user_3, &user_3_add_blocks.last().unwrap().hash(), true);

    // Try to invite team.
    let (nonce_key_pair_seed, domain_indir_invite_block) = indir_invite_domain_block(
        &users[0], TEST_EMAIL_DOMAIN, &user_3_promote_block.hash(), true);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: users[0].client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    // User 1 should be able to accept invite.
    let user_1_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user_1, &domain_indir_invite_block.hash(), true);

    // User 2 should not be able to accept invite.
    let user_2_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user_2, &user_1_accept_invite_block.hash(), false);

    // User 3 should not be able to accept invite.
    let user_3_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user_3, &user_1_accept_invite_block.hash(), false);

    users.push(user_1);
    users.push(user_2);
    users.push(user_3);
    users.push(temp_user);
    blocks.extend(user_2_add_blocks);
    blocks.extend(user_3_add_blocks);
    blocks.push(user_3_promote_block);
    blocks.push(domain_indir_invite_block);
    blocks.push(user_1_accept_invite_block);
    blocks.push(user_2_accept_invite_block);
    blocks.push(user_3_accept_invite_block);
});

gen_test!(member_indir_invite_domain,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Try to invite team.
    let (_nonce_key_pair_seed, domain_indir_invite_block) = indir_invite_domain_block(
        &user, TEST_EMAIL_DOMAIN, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(domain_indir_invite_block);
});

gen_test!(non_member_indir_invite_domain,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Try to invite team.
    let (_nonce_key_pair_seed, domain_indir_invite_block) = indir_invite_domain_block(
        &user, TEST_EMAIL_DOMAIN, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(domain_indir_invite_block);
});

gen_test!(indir_invite_domain_remove_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, domain_add_blocks) = add_domain_indir_blocks(
        &users[0], &user, TEST_EMAIL_DOMAIN, &blocks.last().unwrap().hash());

    // User is removed and then tries to reaccept invite.
    let user_remove_block = remove_user_block(
        &users[0], &user, &domain_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed, &user, &user_remove_block.hash(), false);

    users.push(user);
    users.push(temp_user);
    blocks.extend(domain_add_blocks);
    blocks.push(user_remove_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(indir_invite_domain_leave_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, domain_add_blocks) = add_domain_indir_blocks(
        &users[0], &user, TEST_EMAIL_DOMAIN, &blocks.last().unwrap().hash());

    // User leaves and then reaccepts invite.
    let user_leave_block = leave_team_block(
        &user, &domain_add_blocks.last().unwrap().hash(), true);
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed, &user, &user_leave_block.hash(), true);

    users.push(user);
    users.push(temp_user);
    blocks.extend(domain_add_blocks);
    blocks.push(user_leave_block);
    blocks.push(reaccept_invite_block);
});

gen_test!(indir_invite_domain_reuse,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (indirect invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (temp_user, domain_add_blocks) = add_domain_indir_blocks(
        &users[0], &user, TEST_EMAIL_DOMAIN, &blocks.last().unwrap().hash());

    // User tries to reaccept invite.
    let reaccept_invite_block = accept_indir_invite_block(
        &temp_user.client.sign_key_pair_seed,
        &user,
        &domain_add_blocks.last().unwrap().hash(),
        false,
    );

    users.push(user);
    users.push(temp_user);
    blocks.extend(domain_add_blocks);
    blocks.push(reaccept_invite_block);
});

gen_test!(closed_indir_domain_invite,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (indirect) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (nonce_key_pair_seed, domain_indir_invite_block) = indir_invite_domain_block(
        &users[0], TEST_EMAIL_DOMAIN, &blocks.last().unwrap().hash(), true);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: users[0].client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    // Close all invites.
    let close_invite_block = close_invites_block(
        &users[0], &domain_indir_invite_block.hash(), true);

    // User tries to accept closed invite.
    let user_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &user, &close_invite_block.hash(), false);

    users.push(user);
    users.push(temp_user);
    blocks.push(domain_indir_invite_block);
    blocks.push(close_invite_block);
    blocks.push(user_accept_invite_block);
});

// NOTE: Allowed for sake of consensus.
gen_test!(indir_invite_nonce_accept,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and invite (indirect) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let (nonce_key_pair_seed, user_indir_invite_block) = indir_invite_users_block(
        &users[0], &[&user], &blocks.last().unwrap().hash(), true);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: users[0].client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: user.email.clone(),
    };

    // Temp user accepts the indirect invite.
    let temp_user_accept_invite_block = accept_indir_invite_block(
        &nonce_key_pair_seed, &temp_user, &user_indir_invite_block.hash(), true);

    users.push(user);
    users.push(temp_user);
    blocks.push(user_indir_invite_block);
    blocks.push(temp_user_accept_invite_block);
});
