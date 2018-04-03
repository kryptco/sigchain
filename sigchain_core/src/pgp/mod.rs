use super::Result;
use base64;

extern crate crc24;
extern crate byteorder;

pub fn pgp_public_key_ascii_armor_string(public_key_packets: &[u8]) -> Result<String> {
    let crc = crc24::hash_raw(public_key_packets);
    use self::byteorder::{BigEndian, WriteBytesExt};
    let mut crc_bytes = vec![];
    crc_bytes.write_u24::<BigEndian>(crc)?;

    Ok(format!(
        r"-----BEGIN PGP PUBLIC KEY BLOCK-----
Comment: Created with Krypton

{}
={}
-----END PGP PUBLIC KEY BLOCK-----",
        base64::encode_config(public_key_packets, base64::Config::new(
            base64::CharacterSet::Standard,
            true,
            true,
            base64::LineWrap::Wrap(64, base64::LineEnding::LF),
        )),
        base64::encode(&crc_bytes)))
}

pub fn pgp_signature_ascii_armor_string(signature_packets: &[u8]) -> Result<String> {
    let crc = crc24::hash_raw(signature_packets);
    use self::byteorder::{BigEndian, WriteBytesExt};
    let mut crc_bytes = vec![];
    crc_bytes.write_u24::<BigEndian>(crc)?;

    Ok(format!(
        r"-----BEGIN PGP SIGNATURE-----
Comment: Created with Krypton

{}
={}
-----END PGP SIGNATURE-----",
        base64::encode_config(signature_packets, base64::Config::new(
            base64::CharacterSet::Standard,
            true,
            true,
            base64::LineWrap::Wrap(64, base64::LineEnding::LF),
        )),
        base64::encode(&crc_bytes)))
}

