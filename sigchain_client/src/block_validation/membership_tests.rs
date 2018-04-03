use super::*;

gen_test!(admin_remove_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    // Remove new admin.
    let admin_remove_block = remove_user_block(&users[0], &user, &user_promote_block.hash(), true);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
    blocks.push(admin_remove_block);
});

gen_test!(admin_remove_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Remove self.
    let self_remove_block = remove_user_block(
        &users[0], &users[0], &blocks.last().unwrap().hash(), false);

    blocks.push(self_remove_block);
});

gen_test!(admin_remove_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Remove user.
    let user_remove_block = remove_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_remove_block);
});

gen_test!(admin_remove_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Try to remove user.
    let user_remove_block = remove_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_remove_block);
});

gen_test!(member_remove_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to remove admin.
    let admin_remove_block =
        remove_user_block(&user, &users[0], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_remove_block);
});

gen_test!(member_remove_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate and add (direct invite and accept) user 2 to team.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    // User 1 tries to remove user 2.
    let user_2_remove_block = remove_user_block(
        &user_1, &user_2, &user_2_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
    blocks.push(user_2_remove_block);
});

gen_test!(member_remove_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to remove self.
    let self_remove_block = remove_user_block(
        &user, &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(self_remove_block);
});

gen_test!(member_remove_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to remove user 2.
    let user_2_remove_block = remove_user_block(
        &user_1, &user_2, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_2_remove_block);
});

gen_test!(non_member_remove_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to remove admin.
    let admin_remove_block = remove_user_block(
        &user, &users[0], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(admin_remove_block);
});

gen_test!(non_member_remove_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 2 tries to remove user 1.
    let user_1_remove_block = remove_user_block(
        &user_2, &user_1, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_1_remove_block);
});

gen_test!(non_member_remove_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to remove user 2.
    let user_2_remove_block = remove_user_block(
        &user_1, &user_2, &blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.push(user_2_remove_block);
});

gen_test!(non_member_remove_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to remove self.
    let self_remove_block = remove_user_block(&user, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(self_remove_block);
});

gen_test!(admin_leave_team,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Admin leaves team.
    let admin_leave_block = leave_team_block(&users[0], &blocks.last().unwrap().hash(), true);

    blocks.push(admin_leave_block);
});

gen_test!(member_leave_team,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User leaves team.
    let user_leave_block = leave_team_block(&user, &user_add_blocks.last().unwrap().hash(), true);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_leave_block);
});

gen_test!(non_member_leave_team,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to leaves team.
    let user_leave_block = leave_team_block(&user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_leave_block);
});

gen_test!(admin_promote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    // Try to promote user again.
    let admin_promote_block = promote_user_block(
        &users[0], &user, &user_promote_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
    blocks.push(admin_promote_block);
});

gen_test!(admin_promote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Try to promote self.
    let admin_promote_block = promote_user_block(
        &users[0], &users[0], &blocks.last().unwrap().hash(), false);

    blocks.push(admin_promote_block);
});

gen_test!(admin_promote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
});

gen_test!(admin_promote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Try to promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_promote_block);
});

gen_test!(member_promote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to promote admin.
    let admin_promote_block = promote_user_block(
        &user, &users[0], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_promote_block);
});

gen_test!(member_promote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate and add (direct invite and accept) user 2 to team.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    // User 1 tries to promote user 2.
    let user_2_promote_block = promote_user_block(
        &user_1, &user_2, &user_2_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
    blocks.push(user_2_promote_block);
});

gen_test!(member_promote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to promote self.
    let user_promote_block = promote_user_block(
        &user, &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
});

gen_test!(member_promote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to promote user 2.
    let user_2_promote_block = promote_user_block(
        &user_1, &user_2, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_2_promote_block);
});

gen_test!(non_member_promote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to promote admin.
    let admin_promote_block = promote_user_block(
        &user, &users[0], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(admin_promote_block);
});

gen_test!(non_member_promote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 2 tries to promote user 1.
    let user_1_promote_blocks = promote_user_block(
        &user_2, &user_1, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_1_promote_blocks);
});

gen_test!(non_member_promote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to promote user 2.
    let user_2_promote_block = promote_user_block(
        &user_1, &user_2, &blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.push(user_2_promote_block);
});

gen_test!(non_member_promote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to promote self.
    let user_promote_block = promote_user_block(
        &user, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_promote_block);
});

gen_test!(admin_demote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Promote user to admin.
    let user_promote_block = promote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), true);

    // Demote new admin.
    let admin_demote_block = demote_user_block(
        &users[0], &user, &user_promote_block.hash(), true);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_promote_block);
    blocks.push(admin_demote_block);
});

gen_test!(admin_demote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Demote self.
    let admin_demote_block = demote_user_block(
        &users[0], &users[0], &blocks.last().unwrap().hash(), true);

    blocks.push(admin_demote_block);
});

gen_test!(admin_demote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Try to demote user.
    let user_demote_block = demote_user_block(
        &users[0], &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_demote_block);
});

gen_test!(admin_demote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Try to demote user.
    let user_demote_block = demote_user_block(
        &users[0], &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_demote_block);
});

gen_test!(member_demote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to demote admin.
    let admin_demote_block = demote_user_block(
        &user, &users[0], &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_demote_block);
});

gen_test!(member_demote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate and add (direct invite and accept) user 2 to team.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);
    let user_2_add_blocks = add_user_blocks(
        &users[0], &user_2, &user_1_add_blocks.last().unwrap().hash());

    // User 1 tries to demote user 2.
    let user_2_demote_block = demote_user_block(
        &user_1, &user_2, &user_2_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.extend(user_2_add_blocks);
    blocks.push(user_2_demote_block);
});

gen_test!(member_demote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // User tries to demote self.
    let user_demote_block = demote_user_block(
        &user, &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_demote_block);
});

gen_test!(member_demote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to demote user 2.
    let user_2_demote_block = demote_user_block(
        &user_1, &user_2, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_2_demote_block);
});

gen_test!(non_member_demote_admin,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to demote admin.
    let admin_demote_block = demote_user_block(
        &user, &users[0], &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(admin_demote_block);
});

gen_test!(non_member_demote_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user 1 to team.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);
    let user_1_add_blocks = add_user_blocks(&users[0], &user_1, &blocks.last().unwrap().hash());

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 2 tries to demote user 1.
    let user_1_demote_blocks = demote_user_block(
        &user_2, &user_1, &user_1_add_blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.extend(user_1_add_blocks);
    blocks.push(user_1_demote_blocks);
});

gen_test!(non_member_demote_non_member,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user 1.
    let user_1 = generate_user(&users[0].client.team_public_key, 1);

    // Generate user 2.
    let user_2 = generate_user(&users[0].client.team_public_key, 2);

    // User 1 tries to demote user 2.
    let user_2_demote_block = demote_user_block(
        &user_1, &user_2, &blocks.last().unwrap().hash(), false);

    users.push(user_1);
    users.push(user_2);
    blocks.push(user_2_demote_block);
});

gen_test!(non_member_demote_self,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // User tries to demote self.
    let user_demote_block = demote_user_block(
        &user, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_demote_block);
});
