use super::*;

mod pin;
pub use self::pin::*;

mod add;
pub use self::add::*;

use sshwire::ssh::PublicKeyHeader;

pub fn ssh_public_key_wire_string(public_key_wire: &[u8]) -> Result<String> {
    let header = sshwire::serde_de::from_slice::<PublicKeyHeader>(public_key_wire)?;
    Ok(format!("{} {}", header._type, base64::encode(public_key_wire)))
}
