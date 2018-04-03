use b64data;
use base64;
use chrono::offset::Utc;
use serde_json;
use sha256;

extern crate semver;
use self::semver::Version;

use crypto;
use crypto::{
    SignKeyPair,
    BoxKeyPair,
    sign_keypair_from_seed,
    gen_sign_key_pair_seed,
    gen_sign_key_pair,
    gen_box_key_pair,
};
use errors::Result;
use protocol::*;
use team::Identity;
use Body::*;
use Invitation::*;
use MainChain::*;
use Operation::*;

const TEST_EMAIL_DOMAIN: &str = "invalid";

#[derive(Serialize, Deserialize)]
pub struct BlockValidationTest {

    // The sequence of blocks to add to the team chain with expected results.
    pub blocks: Vec<TestBlock>,

    // The name of test.
    pub name: String,

    // The clients that will participate in the test, verify the sigchain, and achieve consensus.
    pub clients: Vec<Client>,
}

#[derive(Serialize, Deserialize)]
pub struct TestBlock {
    pub signed_message: SignedMessage,
    pub expected: ExpectedResult,
}

impl TestBlock {
    pub fn hash(&self) -> Vec<u8> {
        self.signed_message.payload_hash()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    pub valid: bool,
    #[serde(with = "b64data")]
    pub team_public_key: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    #[serde(with = "b64data")]
    pub sign_key_pair_seed: Vec<u8>,
    pub box_key_pair: BoxKeyPair,
    #[serde(with = "b64data")]
    pub team_public_key: Vec<u8>,
}

#[derive(Clone)]
pub struct User {
    pub client: Client,
    pub sign_key_pair: SignKeyPair,
    pub email: String,
}

macro_rules! gen_test {
    ($name:ident, $gen_data:expr) => {
        pub mod $name {
            use super::*;

            #[test]
            fn test() {
                run_protocol_test(data());
            }

            pub fn data() -> BlockValidationTest {
                let mut blocks = Vec::new();
                let mut users = Vec::new();

                // Set up creator and team.
                let (creator, team_creation_block) = setup_team_block(None);
                blocks.push(team_creation_block);
                users.push(creator);

                // Add the blocks specific to the test.
                $gen_data(&mut users, &mut blocks);

                BlockValidationTest {
                    blocks,
                    name: stringify!($name).into(),
                    clients: users.into_iter().map(|x| x.client).collect::<Vec<Client>>(),
                }
            }
        }
    }
}

mod membership_tests;
use self::membership_tests::*;
mod direct_invite_tests;
use self::direct_invite_tests::*;
mod indirect_invite_tests;
use self::indirect_invite_tests::*;
mod host_pin_tests;
use self::host_pin_tests::*;
mod other_tests;
use self::other_tests::*;

fn block_from_signed_message(signed_message: &SignedMessage, expected: &ExpectedResult) -> TestBlock {
    TestBlock {
        signed_message: signed_message.clone(),
        expected: expected.clone(),
    }
}

#[cfg(test)]
fn run_protocol_test(test: BlockValidationTest) {
    println!("\nRunning test {:?}:", test.name);

    // Form server <-> db connection and create transaction that cannot commit for testing.
    use db;
    let server_conn = db::establish_connection().unwrap();
    server_conn.begin_test_transaction().unwrap();

    // Initialize clients.
    print!("\tCreating local clients...");
    use db::Connection;
    use client::TestClient;
    use std::collections::HashMap;
    let mut key_client_map = HashMap::new();
    for client in test.clients {

        // Create client with local db and create transaction that cannot commit for testing.
        let client_sign_key_pair = sign_keypair_from_seed(&client.sign_key_pair_seed).unwrap();
        let test_client = TestClient::from_key_pair_temp_db(
            client_sign_key_pair.clone(),
            client.box_key_pair.clone(),
            client.team_public_key.clone(),
            &server_conn,
        ).unwrap();
        test_client.db_connection.begin_test_transaction().unwrap();
        key_client_map.insert(client_sign_key_pair.public_key, test_client);
    }
    println!("done");

    // Run the tests.
    println!("\tSending blocks...");
    use client::Client;
    use client::traits::Broadcast;
    for (i, block) in test.blocks.into_iter().enumerate() {
        print!("\tTesting block {}...", i);

        let sending_client = key_client_map.get(&crypto::ed25519::PublicKey::from_slice(
            &block.signed_message.public_key,
        ).unwrap()).unwrap();

        let payload = serde_json::from_str::<Message>(&block.signed_message.message).unwrap().body;
        let server_response = sending_client.broadcast::<E>(&Endpoint::Sigchain, &block.signed_message);

        // TODO: Match response error/types
        assert_eq!(
            block.expected.valid, server_response.is_ok(),
            "Assertion failed by server for block {}: expected {}, received {}. Response: {:?}",
            i, block.expected.valid, server_response.is_ok(), server_response);

        // Update all of the client chains and validate.
        for client in key_client_map.values() {
            let client_response = client.verified_payload_with_db_txn(&block.signed_message);

            if let Body::Main(MainChain::Read(_)) = payload {
                // Client ignores reads.
            } else {
                // TODO: Match response error/types
                assert_eq!(
                    block.expected.valid && block.expected.team_public_key == client.team_public_key, client_response.is_ok(),
                    "Assertion failed by client for block {}: expected {}, received {}. Response: {:?}",
                    i, block.expected.valid && block.expected.team_public_key == client.team_public_key, client_response.is_ok(), client_response);
            };
        }

        println!("done");
    }

    println!("\tDone sending blocks");
    println!("Test {:?} complete", test.name);
}

pub fn generate_identity(sign_key_pair: &SignKeyPair, box_key_pair: &BoxKeyPair, email: &str) -> Identity {
    Identity {
        public_key: sign_key_pair.public_key_bytes().into(),
        encryption_public_key: box_key_pair.public_key_bytes().into(),
        ssh_public_key: Vec::new(),
        pgp_public_key: sign_key_pair.public_key_bytes().into(),
        email: email.into(),
    }
}

pub fn setup_team_block(domain: Option<&str>) -> (User, TestBlock) {
    let (creator, team_creation_msg) = setup_team(domain).unwrap();

    let expected = ExpectedResult {
        valid: true,
        team_public_key: creator.sign_key_pair.public_key_bytes().into(),
    };

    (creator, block_from_signed_message(&team_creation_msg, &expected))
}

pub fn setup_team(domain: Option<&str>) -> Result<(User, SignedMessage)> {
    let creator_sign_key_pair_seed = gen_sign_key_pair_seed()?;
    let creator_sign_key_pair = sign_keypair_from_seed(&creator_sign_key_pair_seed)?;
    let creator_box_key_pair = gen_box_key_pair();
    let team_public_key: Vec<u8> = creator_sign_key_pair.public_key_bytes().into();
    let email_domain = domain.unwrap_or(TEST_EMAIL_DOMAIN);
    let creator = User {
        client: Client {
            sign_key_pair_seed: creator_sign_key_pair_seed,
            box_key_pair: creator_box_key_pair.clone(),
            team_public_key: team_public_key.clone(),
        },
        sign_key_pair: creator_sign_key_pair.clone(),
        email: format!("alex@{}", email_domain),
    };

    let team_creation_msg = SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Create(GenesisBlock {
                team_info: TeamInfo {
                    name: String::from("Acme Engineering"),
                },
                creator_identity: generate_identity(
                    &creator.sign_key_pair,
                    &creator.client.box_key_pair,
                    &creator.email,
                ),
            }))
        },
        &creator.sign_key_pair,
    )?;

    Ok((creator, team_creation_msg))
}

pub fn generate_user(team_public_key: &[u8], member_id: u32) -> User {
    generate_user_with_email(team_public_key, &format!("user_{}_test_{}@{}",
        member_id,
        &base64::encode_config(team_public_key, base64::URL_SAFE)[..8],
        TEST_EMAIL_DOMAIN,
    )).unwrap()
}

pub fn generate_user_with_email(team_public_key: &[u8], email: &str) -> Result<User> {
    let user_sign_key_pair_seed = gen_sign_key_pair_seed()?;
    let user_sign_key_pair = sign_keypair_from_seed(&user_sign_key_pair_seed)?;
    let user_box_key_pair = gen_box_key_pair();

    Ok(User {
        client: Client {
            sign_key_pair_seed: user_sign_key_pair_seed,
            box_key_pair: user_box_key_pair.clone(),
            team_public_key: team_public_key.into(),
        },
        sign_key_pair: user_sign_key_pair.clone(),
        email: email.into(),
    })
}

pub fn dir_invite_user_block(admin: &User, user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&dir_invite_user(admin, user, last_block_hash), &expected)
}

pub fn dir_invite_user(admin: &User, user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Invite(Direct(
                    DirectInvitation{
                        public_key: user.sign_key_pair.public_key_bytes().into(),
                        email: user.email.clone(),
                    },
                )),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn indir_invite_restriction_block(admin: &User, restriction: IndirectInvitationRestriction, last_block_hash: &[u8], valid: bool) -> (Vec<u8>, TestBlock) {
    let (nonce_key_pair_seed, admin_inv_users_msg) = indir_invite_restriction(admin, restriction, last_block_hash);

    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    (nonce_key_pair_seed, block_from_signed_message(&admin_inv_users_msg, &expected))
}

pub fn indir_invite_restriction(admin: &User, restriction: IndirectInvitationRestriction, last_block_hash: &[u8]) -> (Vec<u8>, SignedMessage) {
    let nonce_key_pair_seed = gen_sign_key_pair_seed().unwrap();
    let nonce_key_pair = sign_keypair_from_seed(&nonce_key_pair_seed).unwrap();

    let admin_inv_users_secret = IndirectInvitationSecret {
        initial_team_public_key: admin.client.team_public_key.clone(),
        last_block_hash: last_block_hash.into(),
        nonce_keypair_seed: nonce_key_pair_seed.clone(),
        restriction: restriction.clone(),
    };

    let invite_encryption = crypto::secretbox::ephemeral_encrypt(serde_json::to_vec(
        &admin_inv_users_secret,
    ).unwrap());

    let admin_inv_users_msg = SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Invite(Indirect(
                    IndirectInvitation {
                        nonce_public_key: nonce_key_pair.public_key_bytes().into(),
                        restriction: restriction.clone(),
                        invite_symmetric_key_hash: sha256::hash(
                            &invite_encryption.symmetric_key
                        ).0.as_ref().into(),
                        invite_ciphertext: invite_encryption.nonce_and_ciphertext,
                    },
                )),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap();

    (
        nonce_key_pair_seed,
        admin_inv_users_msg,
    )
}

pub fn indir_invite_users_block(admin: &User, users: &[&User], last_block_hash: &[u8], valid: bool) -> (Vec<u8>, TestBlock) {
    let emails = users.into_iter().map(|user| user.email.clone()).collect::<Vec<String>>();
    indir_invite_restriction_block(
        admin,
        IndirectInvitationRestriction::Emails(emails),
        last_block_hash,
        valid,
    )
}

pub fn indir_invite_users(admin: &User, users: &[&User], last_block_hash: &[u8]) -> (Vec<u8>, SignedMessage) {
    let emails = users.into_iter().map(|user| user.email.clone()).collect::<Vec<String>>();
    indir_invite_restriction(
        admin,
        IndirectInvitationRestriction::Emails(emails),
        last_block_hash,
    )
}

pub fn indir_invite_domain_block(admin: &User, domain: &str, last_block_hash: &[u8], valid: bool) -> (Vec<u8>, TestBlock) {
    indir_invite_restriction_block(
        admin,
        IndirectInvitationRestriction::Domain(domain.into()),
        last_block_hash,
        valid,
    )
}

pub fn indir_invite_domain(admin: &User, domain: &str, last_block_hash: &[u8]) -> (Vec<u8>, SignedMessage) {
    indir_invite_restriction(
        admin,
        IndirectInvitationRestriction::Domain(domain.into()),
        last_block_hash,
    )
}

pub fn accept_dir_invite_block(user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: user.client.team_public_key.clone(),
    };

    block_from_signed_message(&accept_dir_invite(user, last_block_hash), &expected)
}

pub fn accept_dir_invite(user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: AcceptInvite(generate_identity(
                    &user.sign_key_pair,
                    &user.client.box_key_pair,
                    &user.email,
                )),
            })),
        },
        &user.sign_key_pair,
    ).unwrap()
}

pub fn accept_indir_invite_block(nonce_key_pair_seed: &[u8], user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: user.client.team_public_key.clone(),
    };

    block_from_signed_message(&accept_indir_invite(nonce_key_pair_seed, user, last_block_hash), &expected)
}

pub fn accept_indir_invite(nonce_key_pair_seed: &[u8], user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: AcceptInvite(generate_identity(
                    &user.sign_key_pair,
                    &user.client.box_key_pair,
                    &user.email,
                )),
            })),
        },
        &sign_keypair_from_seed(nonce_key_pair_seed).unwrap(),
    ).unwrap()
}

pub fn close_invites_block(admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&close_invites(admin, last_block_hash), &expected)
}

pub fn close_invites(admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: CloseInvitations(E{}),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn add_user_blocks(admin: &User, user: &User, last_block_hash: &[u8]) -> Vec<TestBlock> {
    let expected = ExpectedResult {
        valid: true,
        team_public_key: admin.client.team_public_key.clone(),
    };

    add_user(admin, user, last_block_hash)
        .iter()
        .map(|msg| block_from_signed_message(msg, &expected))
        .collect()
}

pub fn add_user(admin: &User, user: &User, last_block_hash: &[u8]) -> Vec<SignedMessage> {
    let admin_inv_user_msg = dir_invite_user(admin, user, last_block_hash);
    let user_accept_inv_msg = accept_dir_invite(user, &admin_inv_user_msg.payload_hash());

    vec![
        admin_inv_user_msg,
        user_accept_inv_msg,
    ]
}

pub fn add_user_indir_blocks(admin: &User, user: &User, last_block_hash: &[u8]) -> (User, Vec<TestBlock>) {
    let (temp_user, add_user_msgs) = add_user_indir(admin, user, last_block_hash);

    let expected = ExpectedResult {
        valid: true,
        team_public_key: admin.client.team_public_key.clone(),
    };

    let add_user_blocks = add_user_msgs
        .iter()
        .map(|msg| block_from_signed_message(msg, &expected))
        .collect();

    (temp_user, add_user_blocks)
}

pub fn add_user_indir(admin: &User, user: &User, last_block_hash: &[u8]) -> (User, Vec<SignedMessage>) {
    let (nonce_key_pair_seed, indir_invite_user_msg) = indir_invite_users(
        admin, &[user], last_block_hash);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: admin.client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    let accept_invite_msg = accept_indir_invite(
        &nonce_key_pair_seed, user, &indir_invite_user_msg.payload_hash());

    (
        temp_user,
        vec![
            indir_invite_user_msg,
            accept_invite_msg,
        ],
    )
}

pub fn add_domain_indir_blocks(admin: &User, user: &User, domain: &str, last_block_hash: &[u8]) -> (User, Vec<TestBlock>) {
    let (temp_user, add_user_msgs) = add_domain_indir(admin, user, domain, last_block_hash);

    let expected = ExpectedResult {
        valid: true,
        team_public_key: admin.client.team_public_key.clone(),
    };

    let add_user_blocks = add_user_msgs
        .iter()
        .map(|msg| block_from_signed_message(msg, &expected))
        .collect();

    (temp_user, add_user_blocks)
}

pub fn add_domain_indir(admin: &User, user: &User, domain: &str, last_block_hash: &[u8]) -> (User, Vec<SignedMessage>) {
    let (nonce_key_pair_seed, indir_invite_domain_msg) = indir_invite_domain(
        admin, domain, last_block_hash);
    let temp_user = User {
        client: Client {
            sign_key_pair_seed: nonce_key_pair_seed.clone(),
            box_key_pair: gen_box_key_pair(),
            team_public_key: admin.client.team_public_key.clone(),
        },
        sign_key_pair: sign_keypair_from_seed(&nonce_key_pair_seed).unwrap(),
        email: String::from(""),
    };

    let accept_invite_msg = accept_indir_invite(
        &nonce_key_pair_seed, user, &indir_invite_domain_msg.payload_hash());

    (
        temp_user,
        vec![
            indir_invite_domain_msg,
            accept_invite_msg,
        ],
    )
}

pub fn promote_user_block(admin: &User, user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&promote_user(admin, user, last_block_hash), &expected)
}

pub fn promote_user(admin: &User, user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Promote(user.sign_key_pair.public_key_bytes().into()),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn remove_user_block(admin: &User, user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&remove_user(admin, user, last_block_hash), &expected)
}

pub fn remove_user(admin: &User, user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Remove(user.sign_key_pair.public_key_bytes().into()),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn demote_user_block(admin: &User, user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&demote_user(admin, user, last_block_hash), &expected)
}

pub fn demote_user(admin: &User, user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Demote(user.sign_key_pair.public_key_bytes().into()),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn leave_team_block(user: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: user.client.team_public_key.clone(),
    };

    block_from_signed_message(&leave_team(user, last_block_hash), &expected)
}

pub fn leave_team(user: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: Leave(E{}),
            })),
        },
        &user.sign_key_pair,
    ).unwrap()
}

pub fn add_logging_block(admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&add_logging(admin, last_block_hash), &expected)
}

pub fn add_logging(admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: AddLoggingEndpoint(LoggingEndpoint::CommandEncrypted(E{})),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn add_logging_with_version_block(version: Version, admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&add_logging_with_version(version, admin, last_block_hash), &expected)
}

pub fn add_logging_with_version(version: Version, admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header {
                utc_time: Utc::now().timestamp(),
                protocol_version: version,
            },
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: AddLoggingEndpoint(LoggingEndpoint::CommandEncrypted(E{})),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn pin_host_block(host: &str, host_public_key: &[u8], admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&pin_host(host, host_public_key, admin, last_block_hash), &expected)
}

pub fn pin_host(host: &str, host_public_key: &[u8], admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: PinHostKey(SSHHostKey {
                    host: String::from(host),
                    public_key: host_public_key.into(),
                }),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn unpin_host_block(host: &str, host_public_key: &[u8], admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&unpin_host(host, host_public_key, admin, last_block_hash), &expected)
}

pub fn unpin_host(host: &str, host_public_key: &[u8], admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: UnpinHostKey(SSHHostKey {
                    host: String::from(host),
                    public_key: host_public_key.into(),
                }),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn set_policy_block(approval_window: i64, admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&set_policy(approval_window, admin, last_block_hash), &expected)
}

pub fn set_policy(approval_window: i64, admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: SetPolicy(Policy {
                    temporary_approval_seconds: Some(approval_window),
                }),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn set_team_info_block(team_name: &str, admin: &User, last_block_hash: &[u8], valid: bool) -> TestBlock {
    let expected = ExpectedResult {
        valid,
        team_public_key: admin.client.team_public_key.clone(),
    };

    block_from_signed_message(&set_team_info(team_name, admin, last_block_hash), &expected)
}

pub fn set_team_info(team_name: &str, admin: &User, last_block_hash: &[u8]) -> SignedMessage {
    SignedMessage::from_message(
        Message {
            header: Header::new(),
            body: Main(Append(Block {
                last_block_hash: last_block_hash.into(),
                operation: SetTeamInfo(TeamInfo {
                    name: String::from(team_name),
                }),
            })),
        },
        &admin.sign_key_pair,
    ).unwrap()
}

pub fn gather_data() -> Vec<BlockValidationTest> {
    vec![
        create_team::data(),
        admin_remove_admin::data(),
        admin_remove_self::data(),
        admin_remove_member::data(),
        admin_remove_non_member::data(),
        member_remove_admin::data(),
        member_remove_member::data(),
        member_remove_self::data(),
        member_remove_non_member::data(),
        non_member_remove_admin::data(),
        non_member_remove_member::data(),
        non_member_remove_non_member::data(),
        non_member_remove_self::data(),
        admin_leave_team::data(),
        member_leave_team::data(),
        non_member_leave_team::data(),
        admin_promote_admin::data(),
        admin_promote_self::data(),
        admin_promote_member::data(),
        admin_promote_non_member::data(),
        member_promote_admin::data(),
        member_promote_member::data(),
        member_promote_self::data(),
        member_promote_non_member::data(),
        non_member_promote_admin::data(),
        non_member_promote_member::data(),
        non_member_promote_non_member::data(),
        non_member_promote_self::data(),
        admin_demote_admin::data(),
        admin_demote_self::data(),
        admin_demote_member::data(),
        admin_demote_non_member::data(),
        member_demote_admin::data(),
        member_demote_member::data(),
        member_demote_self::data(),
        member_demote_non_member::data(),
        non_member_demote_admin::data(),
        non_member_demote_member::data(),
        non_member_demote_non_member::data(),
        non_member_demote_self::data(),
        admin_dir_invite_admin::data(),
        admin_dir_invite_self::data(),
        admin_dir_invite_member::data(),
        admin_dir_invite_non_member::data(),
        admin_dir_invite_non_member_accept::data(),
        dir_invite_remove_reuse::data(),
        dir_invite_leave_reuse::data(),
        dir_invite_reuse::data(),
        closed_dir_invite::data(),
        member_dir_invite_admin::data(),
        member_dir_invite_member::data(),
        member_dir_invite_self::data(),
        member_dir_invite_non_member::data(),
        non_member_dir_invite_admin::data(),
        non_member_dir_invite_member::data(),
        non_member_dir_invite_non_member::data(),
        non_member_dir_invite_self::data(),
        dir_invite_user_hijack::data(),
        dir_invite_team_hijack::data(),
        dir_invite_accept_id_sig_mismatch_key::data(),
        dir_invite_accept_pk_sig_mismatch_key::data(),
        dir_invite_duplicate::data(),
        admin_indir_invite_admin::data(),
        admin_indir_invite_self::data(),
        admin_indir_invite_member::data(),
        admin_indir_invite_non_member::data(),
        admin_indir_invite_non_member_accept::data(),
        admin_indir_invite_many_non_member::data(),
        admin_indir_invite_mixed::data(),
        indir_invite_remove_reuse::data(),
        indir_invite_leave_reuse::data(),
        indir_invite_reuse::data(),
        closed_indir_invite::data(),
        member_indir_invite_admin::data(),
        member_indir_invite_member::data(),
        member_indir_invite_self::data(),
        member_indir_invite_non_member::data(),
        non_member_indir_invite_admin::data(),
        non_member_indir_invite_member::data(),
        non_member_indir_invite_non_member::data(),
        non_member_indir_invite_self::data(),
        indir_invite_user_hijack::data(),
        indir_invite_team_hijack::data(),
        admin_indir_invite_domain::data(),
        member_indir_invite_domain::data(),
        non_member_indir_invite_domain::data(),
        indir_invite_domain_remove_reuse::data(),
        indir_invite_domain_leave_reuse::data(),
        indir_invite_domain_reuse::data(),
        closed_indir_domain_invite::data(),
        indir_invite_nonce_accept::data(),
        admin_pin_host::data(),
        member_pin_host::data(),
        non_member_pin_host::data(),
        admin_unpin_host::data(),
        member_unpin_host::data(),
        non_member_unpin_host::data(),
        admin_unpin_host_wrong_public_key::data(),
        admin_unpin_host_wrong_host_name::data(),
        admin_duplicate_pin_host::data(),
        admin_pin_many_keys_for_host::data(),
        admin_pin_same_key_for_hosts::data(),
        semver_reject::data(),
        consume_dir_invite::data(),
        create_team_id_sig_mismatch_key::data(),
        create_team_pk_sig_mismatch_key::data(),
        admin_set_policy::data(),
        member_set_policy::data(),
        non_member_set_policy::data(),
        duplicate_set_policy::data(),
        admin_set_team_info::data(),
        member_set_team_info::data(),
        non_member_set_team_info::data(),
        duplicate_set_team_info::data(),
        duplicate_encryption_public_key_on_team::data(),
    ]
}
