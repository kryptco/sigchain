use super::Result;
use super::base64;
use std::process::*;

pub fn local_host_keys(host: &str) -> Result<Vec<Vec<u8>>>{
    let output = Command::new("ssh-keygen")
        .args(&["-F", &host])
        .output()?;
    if !output.status.success() {
        bail!("ssh-keygen error, exit code: {:?}", output.status.code());
    }
    parse_host_keys(&output.stdout)
}

pub fn map_ssh_alias_to_hostname_port(alias: &str) -> Result<String> {
    let output = Command::new("ssh")
        .args(&["-G", "-q", alias])
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        bail!("ssh error, exit code: {:?}", output.status.code());
    }
    let hostname = parse_from_ssh_config(&output.stdout, "hostname")?;
    let port = parse_from_ssh_config(&output.stdout, "port")?;
    Ok(
        match port.as_ref() {
            "22" => hostname,
            _ => format!("[{}]:{}", hostname, port),
        }
    )
}

fn parse_from_ssh_config(config: &[u8], config_key: &str) -> Result<String> {
    String::from_utf8_lossy(config).lines().filter_map::<String, _>(
        |line| {
            let mut toks = line.split_whitespace();
            let (key, val) = (toks.next(), toks.next());
            if key == Some(config_key) {
                val.map(String::from)
            } else {
                None
            }
        }).collect::<Vec<String>>().into_iter()
    .next().ok_or(format!("failed to parse {} from ssh -G", config_key).into())
}

pub fn update_known_hosts(ssh_alias: &str) -> Result<()> {
    let output = Command::new("ssh")
        .args(&[ssh_alias, "-o", "UpdateHostKeys=yes", "exit"])
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        bail!("ssh error, exit code: {:?}", output.status.code());
    }
    Ok(())
}

fn parse_host_keys(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    let keys = String::from_utf8_lossy(bytes);
    Ok(
        keys.lines().filter(|line| !line.starts_with("#")).filter_map(|line| {
            line.split_whitespace().nth(2)
        }).filter_map(|b64_key| {
            base64::decode(b64_key).map(|key| Some(key)).unwrap_or(None)
        }).collect()
        )
}
