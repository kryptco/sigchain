use super::*;

#[cfg(feature = "network_client")]
mod pin;
#[cfg(feature = "network_client")]
pub use self::pin::*;

#[cfg(feature = "network_client")]
mod add;
#[cfg(feature = "network_client")]
pub use self::add::*;

use sshwire::ssh::PublicKeyHeader;

pub fn ssh_public_key_wire_string(public_key_wire: &[u8]) -> Result<String> {
    let header = sshwire::serde_de::from_slice::<PublicKeyHeader>(public_key_wire)?;
    Ok(format!("{} {}", header._type, base64::encode(public_key_wire)))
}
