extern crate iron;
extern crate iron_cors;
use iron_cors::CorsMiddleware;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate hyper;
use std::net;

extern crate dotenv;
use dotenv::dotenv;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

extern crate serde;
#[macro_use]
extern crate serde_derive;

use sigchain_core::b64data;

extern crate sigchain_core;
use sigchain_core::{protocol, db, dashboard_protocol, pgp};
use sigchain_core::errors::Result;
use self::dashboard_protocol::*;

extern crate sigchain_client;
use sigchain_client::{client, ssh};
use client::{DelegatedNetworkClient, Client};

use std::io::Write;

use iron::prelude::*;
use iron::Handler;
use iron::{status};
use iron::modifiers::Header;

use sigchain_core::db::Connection;
use sigchain_core::protocol::SignatureResult;

extern crate includedir;
extern crate phf;
use std::path::Path;
use std::ffi::OsStr;
use iron::headers::{ContentType};
use iron::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use std::sync::Mutex;

use protocol::logs;
use sigchain_client::traits::Identify;
use sigchain_client::traits::DBConnect;

#[cfg(target_os = "linux")]
extern crate openssl_probe;

mod static_files {
    include!(concat!(env!("OUT_DIR"), "/dashboard_frontend.rs"));
}

const DASHBOARD_ASSETS_PATH : &'static str = "../target/deploy-final";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Params {
    pub port: u16,
    #[serde(with="b64data")]
    pub token: Vec<u8>,
}

impl Params {
    fn new_with_port(port: u16) -> Result<Params> {
        Ok(Params {
            port,
            token: sigchain_core::crypto::random_nonce()?,
        })
    }
    fn regenerate_token(&mut self) -> Result<()> {
        self.token = sigchain_core::crypto::random_nonce()?;
        Ok(())
    }
}

fn write_params(params: &Params) -> Result<()> {
    use std::fs::{File, Permissions};
    use std::os::unix::fs::PermissionsExt;

    let params_path = sigchain_client::krd_client::kr_path()?.join("dashboard_params");
    let mut f = File::create(&params_path)?;
    f.set_permissions(Permissions::from_mode(0o600))?;
    f.write_all(&serde_json::to_vec(&params)?)?;
    f.sync_all()?;
    Ok(())
}

use hyper::net::{HttpListener, NetworkListener};
fn listen_reusing_port() -> Result<(Params, HttpListener)> {
    let params_path = sigchain_client::krd_client::kr_path()?.join("dashboard_params");
    use std::fs::{File};

    let params_file = File::open(&params_path)?;
    let mut params = serde_json::from_reader::<_, Params>(params_file)?;
    params.regenerate_token()?;
    write_params(&params)?;

    let tcp_listener = net::TcpListener::bind(&format!("127.0.0.1:{}", params.port))?;
    let listener = HttpListener::from(tcp_listener);
    Ok((params, listener))
}

fn listen_reusing_or_generating_port() -> Result<(Params, HttpListener)> {

    let try_bind_with_existing_parameters = listen_reusing_port();

    let (params, listener) = match try_bind_with_existing_parameters {
        Ok(t) => t,
        Err(e) => {
            info!("Dashboard bind failed or no existing port found, trying new port. error: {}", e);
            let tcp_listener = net::TcpListener::bind("127.0.0.1:0")?;
            let mut listener = HttpListener::from(tcp_listener);
            let params = Params::new_with_port(listener.local_addr()?.port())?;
            write_params(&params)?;
            (params, listener)
        }
    };
    Ok((params, listener))
}

lazy_static! {
    static ref DASHBOARD_HANDLE: Mutex<Option<iron::Listening>> = Mutex::new(None);
}
use std::sync::{Once, ONCE_INIT};

static INIT_LOGGER: Once = ONCE_INIT;

/// Serve local web dashboard
///
/// Port: allocated by OS and stored in params. Reused unless binding to it fails.
///
/// Token: authenticated local clients to the server so that only the current OS user can access it.
/// Regenerated on restart of the server so other OS users cannot phish the token by claiming the
/// same port before the dashboard is re-bound.
pub fn serve_dashboard(reuse_port_only: bool) -> Result<Params> {
    #[cfg(target_os = "linux")]
    {
        openssl_probe::init_ssl_cert_env_vars();
    }
    let mut dashboard_handle = DASHBOARD_HANDLE.lock().unwrap();
    if dashboard_handle.is_some() {
        bail!("Already serving dashboard");
    }

    INIT_LOGGER.call_once(|| {
        let _ = dotenv();
        env_logger::init();
    });


    let (params, mut listener) = if reuse_port_only {
        listen_reusing_port()?
    } else {
        listen_reusing_or_generating_port()?
    };


    use std::collections::hash_set::HashSet;
    use std::iter::FromIterator;
    let cors_middleware = CorsMiddleware::with_whitelist(
        HashSet::<_>::from_iter(
            vec![
                format!("http://localhost:{}", listener.local_addr()?.port()),
            ].into_iter()
        )
    );

    // init the server with a client
    let client = DelegatedNetworkClient::for_current_team()?;
    let server = StaticServer{ params: params.clone(), client: Mutex::new(client) };

    let mut chain = Chain::new(server);
    chain.link_around(cors_middleware);
    if let Ok(listening) = Iron::new(chain).listen(listener, iron::Protocol::http()) {
        info!("Krypton Dashboard listening on http://localhost:{}", listening.socket.port());
        *dashboard_handle = Some(listening);
    }

    Ok(params)
}


struct StaticServer {
    params: Params,
    client: Mutex<DelegatedNetworkClient>,
}

impl Handler for StaticServer {
    fn handle(&self, mut req: &mut Request) -> IronResult<Response> {
        if req.url.path().join("/").starts_with("api") {
            return self.handle_api(&mut req);
        }

        let path = Self::request_path_to_static_file_path(&req.url.path().join("/"));

        match self::static_files::FILES.get(&path) {
            Ok(contents) => {
                match Path::new(&path).extension() {
                    Some(ext) => {
                        if ext == OsStr::new("html") {
                            Ok(Response::with((status::Ok, Header(ContentType::html()), contents.to_vec())))
                        } else if ext == OsStr::new("css") {
                            let css_mime = ContentType(Mime(TopLevel::Text, SubLevel::Css,
                                                            vec![(Attr::Charset, Value::Utf8)]));
                            Ok(Response::with((status::Ok, Header(css_mime), contents.to_vec())))
                        } else if ext == OsStr::new("svg") {
                            let svg_mime = ContentType(Mime(TopLevel::Image, SubLevel::Ext("svg+xml".into()),
                                                            vec![(Attr::Charset, Value::Utf8)]));
                            Ok(Response::with((status::Ok, Header(svg_mime), contents.to_vec())))
                        } else if ext == OsStr::new("wasm") {
                            let wasm_mime = ContentType(Mime(TopLevel::Application, SubLevel::Ext("wasm".into()),
                                                                vec![(Attr::Charset, Value::Utf8)]));
                            Ok(Response::with((status::Ok, Header(wasm_mime), contents.to_vec())))
                        } else {
                            Ok(Response::with((status::Ok, contents.to_vec())))
                        }
                    },
                    None => {
                        Ok(Response::with((status::Ok, contents.to_vec())))
                    }
                }
            },
            Err(_) => Ok(Response::with(status::NotFound)),
        }

    }
}
impl StaticServer {
    fn request_path_to_static_file_path(request_path: &str) -> String {
        let request_path = match request_path {
            "" | "/" => {
                "index.html"
            }
            path => path,
        };
        vec![
            DASHBOARD_ASSETS_PATH,
            request_path,
        ].join("/")
    }
}

header! { (XKryptonDashboardToken, "X-Krypton-Dashboard-Token") => [String] }

impl StaticServer {
    fn handle_api(&self, mut req: &mut Request) -> IronResult<Response> {
        use sigchain_core::base64;
        match req.headers.get::<XKryptonDashboardToken>() {
            Some(ref client_token) => {
                //  Compare hashes to hide timing information
                let client_token = match base64::decode_config(&client_token.0, base64::URL_SAFE) {
                    Ok(client_token) => client_token,
                    _ => {
                        error!("invalid token encoding");
                        return Ok(Response::with(status::Forbidden));
                    }
                };
                use sigchain_core::sha256;
                if sha256::hash(&client_token) != sha256::hash(&self.params.token) {
                    error!("invalid token");
                    return Ok(Response::with(status::Forbidden));
                }
            }
            _ => {
                error!("missing token");
                return Ok(Response::with(status::Forbidden));
            }
        };

        let mut client = self.client.lock().unwrap();

        // check if client's team or identity has changed
        let mut new_client:Option<DelegatedNetworkClient> = None;
        match client.has_team_or_identity_changed() {
            Ok(has_team_or_identity_changed) if has_team_or_identity_changed => {
                match DelegatedNetworkClient::for_current_team() {
                    Ok(the_new_client) => {
                        eprintln!("Reset to a new client");
                        new_client = Some(the_new_client)
                    },
                    Err(error) => {
                        eprintln!("Error resetting client: {:?}", error);
                        return Ok(Response::with(status::InternalServerError));
                    }
                }
            },
            Err(error) => {
                eprintln!("Error checking current team: {:?}", error);
                return Ok(Response::with(status::InternalServerError));
            },
            _ => { }
        };


        let path = req.url.path().join("/");

        if let Some(new_client) = new_client {
            // replace the client
            *client = new_client;
        }

        StaticServer::handle_api_for_path(path, &mut req, &client)
    }

    fn handle_api_for_path(path:String, mut req: &mut Request, client:&DelegatedNetworkClient) -> IronResult<Response> {
        let result = match path.as_str() {
            "api/status" => {
                let status = match krypton_status(client) {
                    Ok(status) => { status }
                    Err(_) => { return Ok(Response::with(status::InternalServerError)); }
                };

                let response = match serde_json::to_vec(&status) {
                    Ok(response) => response,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return Ok(Response::with(status::InternalServerError))
                    },
                };
                return Ok(Response::with((status::Ok, response)));
            }
            "api/demote" => {
                demote_handler(&mut req, client)
            }
            "api/remove" => {
                remove_handler(&mut req, client)
            }
            "api/promote" => {
                promote_handler(&mut req, client)
            }
            "api/team_info" => {
                team_info_handler(&mut req, client)
            }
            "api/policy" => {
                policy_handler(&mut req, client)
            }
            "api/enable_logging" => {
                enable_logging_handler(client)
            }
            "api/disable_logging" => {
                disable_logging_handler()
            }
            "api/invite" => {
                let response = match invite_handler(&mut req, client) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("{:?}", e);
                        return Ok(Response::with(status::InternalServerError))
                    },
                };
                return Ok(Response::with((status::Ok, response)));
            }
            "api/team" => {
                let response = match team_handler(client) {
                    Ok(response) => response,
                    Err(e) => {
                        println!("{:?}", e);
                        return Ok(Response::with(status::InternalServerError))
                    },
                };
                debug!("Bytes returned: {:?}", response.len());
                return Ok(Response::with((status::Ok, response)));
            }
            _ => Ok(()),
        };

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
                match e {
                    sigchain_core::errors::Error(sigchain_core::errors::ErrorKind::Msg(m), _) => {
                        if m == "rejected" {
                            return Ok(Response::with(status::Unauthorized))
                        }
                    }
                    _ => {}
                }
            }
        };

        Ok(Response::with(status::Ok))
    }
}

//  must be in this crate because of sshwire dependency
pub fn to_dashboard_identity(identity: protocol::team::Identity) -> Identity {
    Identity {
        email: identity.email.clone(),
        public_key: identity.public_key.clone(),
        pgp_public_key: pgp::pgp_public_key_ascii_armor_string(&identity.pgp_public_key).unwrap_or("invalid PGP public key".to_string()),
        ssh_public_key: format!("{} {}", ssh::ssh_public_key_wire_string(&identity.ssh_public_key).unwrap_or("invalid SSH public key".to_string()), identity.email.clone()),
    }
}

fn krypton_status(client: &DelegatedNetworkClient) -> Result<KryptonStatus> {
    return match client.get_read_token()? {
        Some(_) => { Ok(KryptonStatus::Approved) }
        _ => { Ok(KryptonStatus::NeedsApproval) }
    };
}

fn team_handler(client: &DelegatedNetworkClient) -> Result<Vec<u8>> {

    let team_db_conn = &client.team_db_conn();

    let team_data_success = client.update_team_blocks()
        .map_err(|e| error!("error updating team blocks: {:?}", e))
        .is_ok();

    let log_blocks_fetch_result = client.update_team_log_blocks_with_limit(Some(5));
    let (log_blocks_success, all_new_logs_loaded) = match log_blocks_fetch_result {
        Ok(has_more) => { (true, has_more) },
        Err(_) => { (false, false) }
    };

    team_db_conn.conn.transaction::<_, sigchain_core::errors::Error, _>(|| {
        let admin_public_keys = db::TeamMembership::filter_admin_public_keys(team_db_conn)?;

        let me = to_dashboard_identity(db::Identity::find(team_db_conn, client.identity_pk())?.into_identity());

        let members = db::Identity::find_all_for_team(team_db_conn)?;
        let mut member_rows:Vec<TeamMember> = members.into_iter().map(
            |m| -> Result<_> {
                let logs: Vec<logs::Log> = db::Log::for_member(team_db_conn, &m.public_key)?
                    .iter().filter_map(|db_log| serde_json::from_str::<logs::Log>(&db_log.log_json).ok()).collect();
                let is_admin = admin_public_keys.contains(&m.public_key);
                let mut sorted_logs = logs.clone();
                sorted_logs.sort_by_key(|log| log.unix_seconds);
                sorted_logs.reverse();

                use std::collections::HashMap;
                let mut logs_by_host = HashMap::new();
                for log in logs.clone() {
                    if let logs::LogBody::Ssh(ssh_log) = log.clone().body {
                        if let Some(host_auth) = ssh_log.host_authorization {
                            logs_by_host.entry(host_auth.host.clone()).or_insert(vec![]).push(log);
                        }
                    }
                }
                let mut hosts = vec![];
                for (host, logs) in logs_by_host {
                    hosts.push(HostAccess{
                        host,
                        accesses: logs.len() as i64,
                        last_access_unix_seconds: logs.clone().iter().max_by_key(|log| log.unix_seconds).map(|log| log.unix_seconds).unwrap_or(0u64) as i64,
                    })
                }
                hosts.sort_by_key(|h| h.host.clone());

                //filter only approved logs.
                let last_access:Option<logs::Log> = logs.clone().iter().filter(|log|
                    log.body.is_success()
                ).max_by_key(|log|
                    log.unix_seconds
                ).cloned();

                use sigchain_core::diesel::OptionalExtension;
                Ok(TeamMember {
                    identity: to_dashboard_identity(m.clone().into_identity()),
                    is_admin,
                    is_removed: db::TeamMembership::find(team_db_conn, &m.public_key).optional()?.is_none(),
                    last_access,
                    logins_today: logs.len() as i64,
                    last_24_hours_accesses: sorted_logs,
                    hosts,
                })
            }
        ).filter_map(|result| result.map_err(|e| {println!("{:?}", e); e}).ok()).collect();

        // sort by logins today
        member_rows.sort_by(|a,b| b.logins_today.cmp(&a.logins_today));

        // move the removed members to the bottom
        member_rows.sort_by(|a,b| a.is_removed.cmp(&b.is_removed));

        let team = db::Team::find(team_db_conn)?;

        let billing_url:String = client.public_key_and_checkpoint.server_endpoints.billing_url(
            &team.name,
            &client.public_key_and_checkpoint.team_public_key,
            &me.public_key,
            &me.email,
        )?;


        //load locally saved billing info
        use protocol::billing::*;
        let billing_info = client.request_billing_info().unwrap_or(BillingInfo {
            current_tier: PaymentTier {
                name: "--".into(),
                price: 0,
                limit: BillingUsage {
                    members: 0,
                    hosts: 0,
                    logs_last_30_days: 1,
                }
            },
            usage: BillingUsage {
                members: member_rows.len().clone() as i64,
                hosts: 0,
                logs_last_30_days: 0,
            }
        });

        let response = DashboardResponse {
            team_name: team.name,
            team_members: member_rows,
            me,
            temporary_approval_seconds: team.temporary_approval_seconds,
            audit_logging_enabled: team.command_encrypted_logging_enabled,
            billing_data: BillingData {
                billing_info,
                url: billing_url,
            },
            data_is_fresh: team_data_success && log_blocks_success,
            all_new_logs_loaded,
        };

        Ok(serde_json::to_vec(&response)?)
    })
}

fn demote_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<()> {
    let demote_request: PublicKeyRequest = serde_json::from_reader(&mut req.body)?;
    client.remove_admin_pk(&demote_request.public_key)
}

fn remove_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<()> {
    let demote_request: PublicKeyRequest = serde_json::from_reader(&mut req.body)?;
    client.remove_member_pk(&demote_request.public_key)
}

fn promote_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<()> {
    let demote_request: PublicKeyRequest = serde_json::from_reader(&mut req.body)?;
    client.add_admin_pk(&demote_request.public_key)
}

fn invite_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<Vec<u8>> {
    let indirect_invite_restriction: protocol::team::IndirectInvitationRestriction = serde_json::from_reader(&mut req.body)?;
    let link = client.create_invite(indirect_invite_restriction)?;

    Ok(serde_json::to_vec(
        &LinkResponse{
            link,
        })?)
}

fn team_info_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<()> {
    use sigchain_core::protocol::team::TeamInfo;
    let team_info: TeamInfo = serde_json::from_reader(&mut req.body)?;
    client.set_team_info(team_info)
}

fn policy_handler(req: &mut Request, client: &DelegatedNetworkClient) -> Result<()> {
    use sigchain_core::protocol::team::Policy;
    let policy: Policy = serde_json::from_reader(&mut req.body)?;
    client.set_policy(policy)
}

fn enable_logging_handler(client: &DelegatedNetworkClient) -> Result<()> {
    client.enable_logging()
}

fn disable_logging_handler() -> Result<()> {
    Ok(())
}
