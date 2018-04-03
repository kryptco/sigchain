use super::*;

gen_test!(admin_pin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);

    blocks.push(admin_pin_host_block);
});

gen_test!(member_pin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let user_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &user, &user_add_blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(user_pin_host_block);
});

gen_test!(non_member_pin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let user_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &user, &blocks.last().unwrap().hash(), false);

    users.push(user);
    blocks.push(user_pin_host_block);
});

gen_test!(admin_unpin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);

    // Unpin host.
    let admin_unpin_host_block = unpin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &admin_pin_host_block.hash(), true);

    blocks.push(admin_pin_host_block);
    blocks.push(admin_unpin_host_block);
});

gen_test!(member_unpin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate and add (direct invite and accept) user to team.
    let user = generate_user(&users[0].client.team_public_key, 1);
    let user_add_blocks = add_user_blocks(&users[0], &user, &blocks.last().unwrap().hash());

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co",
        &host_public_key,
        &users[0],
        &user_add_blocks.last().unwrap().hash(),
        true,
    );

    // User tries to unpin host.
    let user_unpin_host_block = unpin_host_block(
        "test.krypt.co", &host_public_key, &user, &admin_pin_host_block.hash(), false);

    users.push(user);
    blocks.extend(user_add_blocks);
    blocks.push(admin_pin_host_block);
    blocks.push(user_unpin_host_block);
});

gen_test!(non_member_unpin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Generate user.
    let user = generate_user(&users[0].client.team_public_key, 1);

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);

    // User tries to unpin host.
    let user_unpin_host_block = unpin_host_block(
        "test.krypt.co", &host_public_key, &user, &admin_pin_host_block.hash(), false);

    users.push(user);
    blocks.push(admin_pin_host_block);
    blocks.push(user_unpin_host_block);
});

gen_test!(admin_unpin_host_wrong_public_key,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);

    // Try to unpin different public key from host name.
    let new_host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_unpin_host_block = unpin_host_block(
        "test.krypt.co", &new_host_public_key, &users[0], &admin_pin_host_block.hash(), false);

    blocks.push(admin_pin_host_block);
    blocks.push(admin_unpin_host_block);
});

gen_test!(admin_unpin_host_wrong_host_name,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin host.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);

    // Try to unpin same public key from non-existent host name.
    let admin_unpin_host_block = unpin_host_block(
        "test-2.krypt.co", &host_public_key, &users[0], &admin_pin_host_block.hash(), false);

    blocks.push(admin_pin_host_block);
    blocks.push(admin_unpin_host_block);
});

gen_test!(admin_duplicate_pin_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin host and try to pin it again.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);
    let admin_pin_host_again_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &admin_pin_host_block.hash(), false);

    blocks.push(admin_pin_host_block);
    blocks.push(admin_pin_host_again_block);
});

gen_test!(admin_pin_many_keys_for_host,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin many keys to a host.
    let host_public_key_1: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let host_public_key_2: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_1_block = pin_host_block(
        "test.krypt.co", &host_public_key_1, &users[0], &blocks.last().unwrap().hash(), true);
    let admin_pin_host_2_block = pin_host_block(
        "test.krypt.co", &host_public_key_2, &users[0], &admin_pin_host_1_block.hash(), true);

    blocks.push(admin_pin_host_1_block);
    blocks.push(admin_pin_host_2_block);
});

gen_test!(admin_pin_same_key_for_hosts,
|users: &mut Vec<User>, blocks: &mut Vec<TestBlock>| {

    // Pin same key to many hosts.
    let host_public_key: Vec<u8> = gen_sign_key_pair().unwrap().public_key_bytes().into();
    let admin_pin_host_1_block = pin_host_block(
        "test.krypt.co", &host_public_key, &users[0], &blocks.last().unwrap().hash(), true);
    let admin_pin_host_2_block = pin_host_block(
        "test-2.krypt.co", &host_public_key, &users[0], &admin_pin_host_1_block.hash(), true);

    blocks.push(admin_pin_host_1_block);
    blocks.push(admin_pin_host_2_block);
});
