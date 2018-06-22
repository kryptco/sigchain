pub use super::sodiumoxide::crypto::box_::curve25519xsalsa20poly1305 as ed25519_box;
use super::sodiumoxide::init;

#[derive(Serialize, Deserialize, Clone)]
pub struct BoxKeyPair {
    #[serde(with="serde_publickey")]
    pub public_key: ed25519_box::PublicKey,
    #[serde(with="serde_secretkey")]
    pub secret_key: ed25519_box::SecretKey,
}

impl BoxKeyPair {
    pub fn public_key_bytes(&self) -> &[u8] {
        use std::borrow::Borrow;
        self.public_key.0.borrow()
    }
}

pub fn seal(plaintext: &[u8], secret_key: &ed25519_box::SecretKey, public_key: &ed25519_box::PublicKey) -> Result<Vec<u8>> {
    init()?;
    let nonce = ed25519_box::gen_nonce();
    let ciphertext = ed25519_box::seal(&plaintext, &nonce, public_key, secret_key);

    let nonce_and_ciphertext : Vec<u8> = [nonce.0.as_ref().into(), ciphertext.as_slice()].concat();
    Ok(nonce_and_ciphertext)
}

pub fn open(nonce_and_ciphertext: &[u8], sender_public_key: &[u8], recipient_secret_key: &ed25519_box::SecretKey) -> Result<Vec<u8>> {
    if nonce_and_ciphertext.len() < ed25519_box::NONCEBYTES {
        bail!("ciphertext too short")
    }
    let checked_nonce = ed25519_box::Nonce::from_slice(&nonce_and_ciphertext[0..ed25519_box::NONCEBYTES])
        .ok_or("invalid nonce")?;
    let checked_sender_pk = ed25519_box::PublicKey::from_slice(sender_public_key).ok_or("invalid key")?;

    Ok(ed25519_box::open(&nonce_and_ciphertext[ed25519_box::NONCEBYTES..], &checked_nonce, &checked_sender_pk, recipient_secret_key)
        .map_err(|_| "box::open failed")?)
}

mod serde_publickey {
    use super::*;
    use {base64, serde};
    use serde::{Serializer, Deserialize, Deserializer};
    use std::ops::Index;
    use std::ops::RangeFull;
    use std::result::Result;
    pub fn serialize<S, T>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
              T: Index<RangeFull, Output = [u8]>
    {
        serializer.serialize_str(&base64::encode(&data[..]))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ed25519_box::PublicKey, D::Error>
        where D: Deserializer<'de>
    {
        let b64 = String::deserialize(deserializer)?;
        let bytes = base64::decode(&b64).map_err(serde::de::Error::custom)?;
        Ok(
            ed25519_box::PublicKey::from_slice(&bytes)
                .ok_or(serde::de::Error::custom("ed25519::PublicKey::from_slice failed"))?
        )
    }
}

mod serde_secretkey {
    use super::*;
    use {base64, serde};
    use serde::{Serializer, Deserialize, Deserializer};
    use std::ops::Index;
    use std::ops::RangeFull;
    use std::result::Result;
    pub fn serialize<S, T>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
              T: Index<RangeFull, Output = [u8]>
    {
        serializer.serialize_str(&base64::encode(&data[..]))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ed25519_box::SecretKey, D::Error>
        where D: Deserializer<'de>
    {
        let b64 = String::deserialize(deserializer)?;
        let bytes = base64::decode(&b64).map_err(serde::de::Error::custom)?;
        Ok(
            ed25519_box::SecretKey::from_slice(&bytes)
                .ok_or(serde::de::Error::custom("ed25519::SecretKey::from_slice failed"))?
        )
    }
}

use super::Result;

pub fn gen_box_key_pair() -> super::Result<BoxKeyPair> {
    init()?;
    let (public_key, secret_key) = ed25519_box::gen_keypair();
    Ok(BoxKeyPair{
        public_key,
        secret_key,
    })
}
