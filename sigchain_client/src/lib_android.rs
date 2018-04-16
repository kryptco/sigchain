use super::*;

use sigchain_core::jni;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JString};
use self::jni::sys::{jstring};
use std::ffi::{CStr,};

use std::sync::Arc;

use std::os::raw::c_void;
use std::os::raw::c_float;
use std::os::raw::c_double;
use std::os::raw::c_char;
use std::os::raw::c_schar;
use std::os::raw::c_uchar;
use std::os::raw::c_int;
use std::os::raw::c_short;
use std::os::raw::c_ushort;
use std::os::raw::c_longlong;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NativeResult<T> {
    Success(T),
    Error(String),
}

use client::traits::{DBConnect, Identify, Broadcast};
use client::OwnedKeyPair;

#[derive(Serialize, Deserialize, Debug)]
struct CreateTeamArgs {
    name: String,
    enable_audit_logs: bool,
    creator_profile: enclave_protocol::Profile,
    temporary_approval_seconds: i64,
    pinned_hosts: Vec<SSHHostKey>,
    email_challenge_nonce: String,
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_createTeam(
    env: JNIEnv, _ : JClass,
    dir: JString,
    create_team_args_json: JString,
) -> jstring {
    android_wrapper(&env, || -> Result<E> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();

        let create_team_args_json = env.get_string(create_team_args_json)?;
        let create_team_args_json : String = create_team_args_json.into();
        let create_team_args : CreateTeamArgs = serde_json::from_str(&create_team_args_json)?;

        let email_challenge_nonce =
            base64::decode_config(&create_team_args.email_challenge_nonce, base64::URL_SAFE)?;

        let sign_key_pair = gen_sign_key_pair()?;
        let box_key_pair = gen_box_key_pair();
        let pk : Vec<u8> = sign_key_pair.public_key_bytes().into();

        let creator_profile = &create_team_args.creator_profile;
        let admin_identity = Identity{
            public_key: pk.clone(),
            encryption_public_key: box_key_pair.public_key_bytes().into(),
            ssh_public_key: creator_profile.ssh_wire_public_key.clone(),
            pgp_public_key: creator_profile.pgp_public_key.clone().ok_or("no PGP public key")?,
            email: creator_profile.email.clone(),
        };
        let create_chain = GenesisBlock {
            team_info: TeamInfo {
                name: create_team_args.name.clone(),
            },
            creator_identity: admin_identity,
        };
        let create_chain_message = Message::new(Body::Main(MainChain::Create(create_chain.clone())));
        let signed_genesis_block = SignedMessage::from_message(create_chain_message, &sign_key_pair)?;

        let cli = AndroidClient{
            key_pair: sign_key_pair,
            box_key_pair: box_key_pair.clone(),
            team_checkpoint: TeamCheckpoint {
                public_key: pk.clone(),
                team_public_key: pk.clone(),
                last_block_hash: signed_genesis_block.payload_hash(),
                server_endpoints: SERVER_ENDPOINTS.lock().unwrap().clone(),
            },
            http_client: get_shared_http_client()?,
            db_connection: AndroidClient::db_conn(dir.clone())?,
        };

        cli.verify_email(&creator_profile.email, &email_challenge_nonce)?;

        cli.commit_send::<E>(&protocol::Endpoint::Sigchain, &signed_genesis_block)?;
        db::CurrentTeam{
            team_checkpoint: serde_json::to_vec(&TeamCheckpoint{
                public_key: pk.clone(),
                team_public_key: pk.clone(),
                last_block_hash: cli.get_last_block_hash()?.ok_or("no last_block_hash")?,
                server_endpoints: SERVER_ENDPOINTS.lock().unwrap().clone(),
            })?,
            sign_key_pair: Some(serde_json::to_vec(&cli.key_pair)?),
            box_key_pair: Some(serde_json::to_vec(&box_key_pair)?),
        }.set(cli.db_conn())?;

        cli.set_policy(Policy{temporary_approval_seconds: Some(create_team_args.temporary_approval_seconds)})?;
        for pinned_host in create_team_args.pinned_hosts {
            debug_log(&format!("pinning {:?}", &pinned_host));
            cli.pin_host_key(&pinned_host.host, &pinned_host.public_key)?;
        }
        if create_team_args.enable_audit_logs {
            cli.enable_logging()?;
        }
        Ok(E{})
    })
}

#[derive(Serialize, Deserialize, Clone)]
struct GenerateClientInput {
    #[serde(with = "b64data")]
    team_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    last_block_hash: Vec<u8>,
    profile: Profile,
}

/// Create a client and identity that will later be used to accept an invitation
#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_generateClient(
    env: JNIEnv, _ : JClass,
    dir: JString,
    args: JString,
) -> jstring {
    time_fn!("generateClient");
    android_wrapper(&env, || -> Result<_> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();

        let args = env.get_string(args)?;
        let args : String = args.into();

        let args : GenerateClientInput = serde_json::from_str(&args)?;
        let profile = args.profile;

        let sign_kp = gen_sign_key_pair()?;
        let box_kp = gen_box_key_pair();

        let conn = &AndroidClient::db_conn(dir.clone())?;
        db::CurrentTeam{
            team_checkpoint: serde_json::to_vec(&TeamCheckpoint{
                public_key: sign_kp.public_key_bytes().into(),
                team_public_key: args.team_public_key,
                last_block_hash: args.last_block_hash,
                server_endpoints: SERVER_ENDPOINTS.lock().unwrap().clone(),
            })?,
            sign_key_pair: Some(serde_json::to_vec(&sign_kp)?),
            box_key_pair: Some(serde_json::to_vec(&box_kp)?),
        }.set(conn)?;

        Ok(Identity{
            public_key: sign_kp.public_key_bytes().into(),
            encryption_public_key: box_kp.public_key_bytes().into(),
            ssh_public_key: profile.ssh_wire_public_key.clone(),
            pgp_public_key: profile.pgp_public_key.clone().ok_or("no PGP public key")?,
            email: profile.email,
        })
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_requestEmailChallenge(
    env: JNIEnv, _ : JClass,
    dir: JString,
    email: JString,
) -> jstring {
    time_fn!("requestEmailChallenge");
    android_wrapper(&env, || -> Result<E> {

        let email = env.get_string(email)?;
        let email : String = email.into();
        AndroidClient::request_email_challenge(
            get_shared_http_client()?,
            &SERVER_ENDPOINTS.lock().unwrap().clone(),
            &email::PutSendEmailChallengeRequest{
                email,
            })?;

        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_deleteDB(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("deleteDB");
    android_wrapper(&env, || -> Result<E> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();
        use std::fs;
        fs::remove_file(dir + "/team.db")?;
        Ok(E{})
    })
}

#[derive(Serialize, Deserialize, Debug)]
struct Member {
    identity: Identity,
    is_admin: bool,
    is_removed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamHomeData {
    #[serde(with = "b64data")]
    team_public_key: Vec<u8>,
    #[serde(with = "b64data")]
    last_block_hash: Vec<u8>,
    #[serde(with = "b64data")]
    identity_public_key: Vec<u8>,
    name: String,
    email: String,
    is_admin: bool,
    n_open_invites: u64,
    n_blocks: u64,
    members: Vec<Member>,
    audit_logs_enabled: bool,
    temporary_approval_seconds: Option<i64>,
    pinned_hosts: Vec<SSHHostKey>,
    billing_url: Option<String>,
}

fn load_team_home_data(dir: &str) -> Result<TeamHomeData> {
    time_fn!("load_team_home_data");
    let conn = AndroidClient::db_conn(dir.into())?;
    let current_team = match db::CurrentTeam::find(&conn).optional()? {
        Some(current_team) => current_team,
        None => bail!("no current team"),
    };
    let team_checkpoint : TeamCheckpoint = serde_json::from_slice(&current_team.team_checkpoint)?;

    let cli = AndroidClient{
        key_pair: serde_json::from_slice(&current_team.sign_key_pair.ok_or("no sign_key_pair")?)?,
        box_key_pair: serde_json::from_slice(&current_team.box_key_pair.ok_or("no box_key_pair")?)?,
        team_checkpoint: team_checkpoint.clone(),
        db_connection: conn,
        http_client: get_shared_http_client()?,
    };

    let team_conn = &db::TeamDBConnection{team: cli.team_pk(), conn: cli.db_conn()};

    use itertools;

    let members = itertools::process_results(
        cli.get_active_and_removed_members()?.into_iter().map(|i| -> Result<Member> {
            let optional_membership = db::TeamMembership::find(team_conn, &i.public_key).optional()?;
            Ok(Member {
                is_admin: optional_membership.as_ref().map(|m| m.is_admin).unwrap_or(false),
                is_removed: optional_membership.is_none(),
                identity: i,
            })
        }),
        |i| i.collect::<Vec<_>>(),
    )?;


    Ok(TeamHomeData{
        team_public_key: cli.team_pk().into(),
        identity_public_key: cli.identity_pk().into(),
        last_block_hash: team_checkpoint.last_block_hash,
        email: cli.get_my_identity()?.email,
        is_admin: cli.is_admin()?,
        members,
        pinned_hosts: cli.get_all_pinned_host_keys()?,
        name: cli.get_team_info()?.name,
        temporary_approval_seconds: cli.get_policy()?.temporary_approval_seconds,
        n_open_invites: db::IndirectInvitation::count(team_conn)? + db::DirectInvitation::count(team_conn)?,
        n_blocks: cli.main_chain_block_count()?,
        audit_logs_enabled: cli.is_command_encrypted_logging_enabled()?,
        billing_url: cli.get_billing_url().ok(),
    })

}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_getTeam(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("getTeam");
    android_wrapper(&env, || -> Result<TeamHomeData> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();

        load_team_home_data(&dir)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_getTeamCheckpoint(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("getTeamCheckpoint");
    android_cli_wrapper(&env, dir, |cli| -> Result<TeamCheckpoint> {
        let current_team = match db::CurrentTeam::find(&cli.db_conn()).optional()? {
            Some(current_team) => current_team,
            None => bail!("no current team"),
        };
        Ok(serde_json::from_slice(&current_team.team_checkpoint)?)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_getPolicy(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("getPolicy");
    android_cli_wrapper(&env, dir, |cli| -> Result<team::Policy> {
        let conn = &db::TeamDBConnection{team: cli.team_pk(), conn: cli.db_conn()};

        Ok(team::Policy{
            temporary_approval_seconds: db::Team::find(conn)?.temporary_approval_seconds,
        })
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_getPinnedKeysByHost(
    env: JNIEnv, _ : JClass,
    dir: JString,
    host: JString,
) -> jstring {
    time_fn!("getPinnedKeysByHost");
    android_cli_wrapper(&env, dir, |cli| -> Result<Vec<team::SSHHostKey>> {
        let host = env.get_string(host)?;
        let host : String = host.into();

        let conn = &db::TeamDBConnection{team: cli.team_pk(), conn: cli.db_conn()};
        Ok(db::PinnedHostKey::filter_by_host(conn.conn, conn.team, &host, false)?
            .into_iter().map(db::PinnedHostKey::into).collect::<Vec<_>>())
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UpdateTeamOutput {
    last_formatted_block: Option<format_blocks::FormattedBlock>,
    more: bool,
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_updateTeam(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("updateTeam");
    android_cli_wrapper(&env, dir, |cli| -> Result<UpdateTeamOutput> {
        let b = cli.read_next_block()?;
        let mut last_formatted_block = None;
        if let Some(last_block) = b.blocks.iter().last() {
            last_formatted_block = Some(format_blocks::format(cli, last_block)?);
        }
        Ok(UpdateTeamOutput{
            last_formatted_block,
            more: b.more,
        })
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_executeRequestableOperation(
    env: JNIEnv, _ : JClass,
    dir: JString,
    operation_json: JString,
) -> jstring {
    time_fn!("executeRequestableOperation");
    android_cli_wrapper(&env, dir, |cli| -> Result<TeamOperationResponse> {
        let operation_json = env.get_string(operation_json)?;
        let operation_json : String = operation_json.into();
        let operation : RequestableTeamOperation = serde_json::from_str(&operation_json)?;

        let result = cli.execute_requestable_operation(&operation)?;

        if let RequestableTeamOperation::Leave(_) = operation {
            use std::fs;
            fs::remove_file(AndroidClient::db_dir_to_db_file(env.get_string(dir)?.to_str()?))?;
        }

        Ok(result)
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecryptInviteOutput {
    pub indirect_invitation_secret: IndirectInvitationSecret,
    pub team_name: String,
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_decryptInvite(
    env: JNIEnv, _ : JClass,
    dir: JString,
    invite_link: JString,
) -> jstring {
    time_fn!("decryptInvite");
    android_wrapper(&env, || -> Result<DecryptInviteOutput> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();

        let invite_link = env.get_string(invite_link)?;
        let invite_link : String = invite_link.into();


        if load_team_home_data(&dir).is_ok() {
            bail!("already on a team")
        }

        let conn = &AndroidClient::db_conn(dir.clone())?;


        let invitation_secret = AndroidClient::fetch_and_decrypt_invite_ciphertext(
            get_shared_http_client()?,
            &SERVER_ENDPOINTS.lock().unwrap().clone(),
            &invite_link,
        )?;

        let server_endpoints = SERVER_ENDPOINTS.lock().unwrap().clone();
        let cli = AndroidClient::bootstrap_using_invite(
            get_shared_http_client()?,
            server_endpoints.clone(),
            dir.clone(),
            invitation_secret.clone(),
        )?;

        db::CurrentTeam{
            team_checkpoint: serde_json::to_vec(&TeamCheckpoint{
                public_key: cli.identity_pk().into(),
                team_public_key: cli.team_pk().into(),
                last_block_hash: cli.get_last_block_hash()?.ok_or("no last_block_hash")?,
                server_endpoints: server_endpoints.clone(),
            })?,
            sign_key_pair: Some(serde_json::to_vec(&cli.key_pair)?),
            box_key_pair: Some(serde_json::to_vec(&cli.box_key_pair)?),
        }.set(cli.db_conn())?;

        Ok(DecryptInviteOutput{
                indirect_invitation_secret: invitation_secret.clone(),
                team_name: cli.get_team_info()?.name,
        })
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AcceptInviteArgs {
    profile: Profile,
    #[serde(with = "b64data")]
    email_challenge_nonce: Vec<u8>,
    invite_secret: IndirectInvitationSecret,
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_acceptInvite(
    env: JNIEnv, _ : JClass,
    dir: JString,
    args: JString,
) -> jstring {
    time_fn!("acceptInvite");
    android_cli_wrapper(&env, dir, |cli| -> Result<E> {
        let args = env.get_string(args)?;
        let args : String = args.into();
        let args : AcceptInviteArgs = serde_json::from_str(&args)?;

        cli.accept_invite(
            Identity{
                email: args.profile.email,
                public_key: cli.identity_pk().into(),
                encryption_public_key: cli.box_public_key().0.to_vec(),
                ssh_public_key: args.profile.ssh_wire_public_key,
                pgp_public_key: args.profile.pgp_public_key.ok_or("no PGP public key")?,
            },
            args.email_challenge_nonce,
            args.invite_secret,
        )?;

        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_tryRead(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("tryRead");
    android_cli_wrapper(&env, dir, |cli| -> Result<E> {
        let dir = env.get_string(dir)?;
        let dir : String = dir.into();

        cli.read_next_block()?;

        Ok(E{})
    })
}

#[derive(Serialize, Deserialize)]
struct AcceptDirectInviteArgs {
    #[serde(with = "b64data")]
    email_challenge_nonce: Vec<u8>,
    identity: Identity,
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_acceptDirectInvite(
    env: JNIEnv, _ : JClass,
    dir: JString,
    args: JString,
) -> jstring {
    time_fn!("acceptDirectInvite");
    android_cli_wrapper(&env, dir, |cli| -> Result<E> {
        let args = env.get_string(args)?;
        let args: String = args.into();
        let args: AcceptDirectInviteArgs = serde_json::from_str(&args)?;

        cli.update_team_blocks()?;
        cli.accept_direct_invite(&args.identity, &args.email_challenge_nonce)?;
        
        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_encryptLog(
    env: JNIEnv, _ : JClass,
    dir: JString,
    log_json: JString,
) -> jstring {
    time_fn!("encryptLog");
    android_cli_wrapper(&env, dir, |cli| -> Result<E> {
        let log_json = env.get_string(log_json)?;
        let log_json: String = log_json.into();

        let log : logs::Log = serde_json::from_str(&log_json)?;

        if let Err(e) = cli.encrypt_log(log) {
            //  Try updating chain from server in case this client crashed before saving a block that was sent to the server
            cli.update_my_log_blocks()?;
        }
        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_formatBlocks(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("formatBlocks");
    use format_blocks;
    android_cli_wrapper(&env, dir, |cli| -> Result<Vec<format_blocks::FormattedBlock>> {

        let conn = &db::TeamDBConnection{team: cli.team_pk(), conn: cli.db_conn()};

        let mut formatted_blocks =
            db::Block::collect_all(conn)?.into_iter()
            .filter_map(|b|
                format_blocks::format(cli, &SignedMessage{
                    public_key: b.member_public_key,
                    message: b.operation,
                    signature: b.signature,
                }).ok()
            ).collect::<Vec<_>>();

        //  Change to reverse-chronological order
        formatted_blocks.reverse();
        Ok(formatted_blocks)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_formatRequestableOp(
    env: JNIEnv, _ : JClass,
    dir: JString,
    requestable_op_json: JString,
) -> jstring {
    time_fn!("formatRequestableOp");
    use format_blocks;
    android_cli_wrapper(&env, dir, |cli| -> Result<format_blocks::FormattedRequestableOperation> {
        let requestable_op_json = env.get_string(requestable_op_json)?;
        let requestable_op_json: String = requestable_op_json.into();

        let op : enclave_protocol::RequestableTeamOperation = serde_json::from_str(&requestable_op_json)?;

        format_blocks::format_requestable_op(cli, op)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_subscribeToPushNotifications(
    env: JNIEnv, _ : JClass,
    dir: JString,
    device_token: JString,
) -> jstring {
    use format_blocks;
    android_cli_wrapper(&env, dir, |cli| -> Result<E> {
        let device_token = env.get_string(device_token)?;
        let device_token: String = device_token.into();

        cli.subscribe_to_push_notifications(push::PushDevice::Android(device_token))?;

        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_signReadToken(
    env: JNIEnv, _ : JClass,
    dir: JString,
    reader_public_key_b64: JString,
) -> jstring {
    time_fn!("signReadToken");
    android_cli_wrapper(&env, dir, |cli| -> Result<team::SignedReadToken> {
        let reader_public_key_b64 = env.get_string(reader_public_key_b64)?;
        let reader_public_key_b64: String = reader_public_key_b64.into();

        let reader_public_key = base64::decode(&reader_public_key_b64)?;

        cli.sign_read_token(&reader_public_key)
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_unwrapKey(
    env: JNIEnv, _ : JClass,
    dir: JString,
    boxed_message_json: JString,
) -> jstring {
    time_fn!("unwrapKey");
    android_cli_wrapper(&env, dir, |cli| -> Result<enclave_protocol::LogDecryptionResponse> {
        let boxed_message_json = env.get_string(boxed_message_json)?;
        let boxed_message_json: String = boxed_message_json.into();

        let boxed_message : logging::BoxedMessage = serde_json::from_str(&boxed_message_json)?;

        Ok(enclave_protocol::LogDecryptionResponse {
            log_decryption_key: cli.unwrap_log_encryption_key(&boxed_message)?,
        })
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_requestBillingInfo(
    env: JNIEnv, _ : JClass,
    dir: JString,
) -> jstring {
    time_fn!("requestBillingInfo");
    android_cli_wrapper(&env, dir, |cli| -> Result<billing::BillingInfo> {
        cli.request_billing_info()
    })
}

use std::sync::Mutex;

lazy_static! {
    static ref SERVER_ENDPOINTS: Mutex<ServerEndpoints> = Mutex::new(ServerEndpoints::prod());
    static ref SHARED_HTTP_CLIENT: Mutex<Option<Arc<reqwest::Client>>> = Mutex::new(None);
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_setProd(
    env: JNIEnv, _ : JClass,
) -> jstring {
    android_wrapper(&env, || -> Result<E> {
        *SERVER_ENDPOINTS.lock().unwrap() = ServerEndpoints::prod();
        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_setStaging(
    env: JNIEnv, _ : JClass,
) -> jstring {
    android_wrapper(&env, || -> Result<E> {
        *SERVER_ENDPOINTS.lock().unwrap() = ServerEndpoints::staging();
        Ok(E{})
    })
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_krypt_krypton_team_Native_setDev(
    env: JNIEnv, _ : JClass,
) -> jstring {
    android_wrapper(&env, || -> Result<E> {
        *SERVER_ENDPOINTS.lock().unwrap() = ServerEndpoints::dev();
        Ok(E{})
    })
}

fn android_wrapper<T: serde::Serialize, F: FnOnce() -> Result<T>>(
    env: &JNIEnv,
    f: F) -> jstring {

    let res = f().map_err(|e| {
        let err = format!("{:?}", e);
        debug_log(&err);
        e
    });

    let wrapped_res = match res {
        Ok(s) => NativeResult::Success(s),
        Err(e) => NativeResult::Error(format!("{}", e)),
    };

    let output = env.new_string(serde_json::to_string(&wrapped_res).unwrap_or("failed to serialize NativeResult".to_string()));

    output.expect("failed to create JVM string").into_inner()
}

fn get_shared_http_client() -> Result<Arc<reqwest::Client>> {
    use std::time::Duration;
    let mut http_client = SHARED_HTTP_CLIENT.lock().unwrap();
    if http_client.is_none() {
        *http_client = Some(Arc::new(reqwest::ClientBuilder::new()?.timeout(Duration::from_secs(10)).build()?));
    }

    let http_client = http_client.as_ref().ok_or("http_client not initialized")?;
    Ok(http_client.clone())
}

fn android_cli_wrapper<T: serde::Serialize, F: FnOnce(&AndroidClient) -> Result<T>>(
    env: &JNIEnv,
    dir: JString,
    f: F) -> jstring {
    android_wrapper(env, || -> Result<T> {
        let dir = env.get_string(dir)?;
        let dir: String = dir.into();

        let conn = AndroidClient::db_conn(dir.clone())?;

        let current_team = match db::CurrentTeam::find(&conn).optional()? {
            Some(current_team) => current_team,
            None => bail!("no current team"),
        };
        let team_checkpoint : TeamCheckpoint = serde_json::from_slice(&current_team.team_checkpoint)?;

        let cli = AndroidClient{
            key_pair: serde_json::from_slice(&current_team.sign_key_pair.ok_or("no sign_key_pair")?)?,
            box_key_pair: serde_json::from_slice(&current_team.box_key_pair.ok_or("no box_key_pair")?)?,
            team_checkpoint,
            http_client: get_shared_http_client()?,
            db_connection: conn,
        };

        f(&cli)
    })
}
