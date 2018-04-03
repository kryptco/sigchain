pub use super::sodiumoxide::crypto::sign::ed25519;

#[derive(Serialize, Deserialize, Clone)]
pub struct SignKeyPair {
    #[serde(with="serde_publickey")]
    pub public_key: ed25519::PublicKey,
    #[serde(with="serde_secretkey")]
    pub secret_key: ed25519::SecretKey,
}

impl SignKeyPair {
    pub fn public_key_bytes(&self) -> &[u8] {
        use std::borrow::Borrow;
        self.public_key.0.borrow()
    }
    pub fn public_key_base64_string(&self) -> String {
        use base64;
        base64::encode_config(self.public_key_bytes(), base64::URL_SAFE)
    }
}

pub fn sign_keypair_from_seed(seed: &[u8]) -> Result<SignKeyPair> {
    let seed = match ed25519::Seed::from_slice(seed) {
        Some(seed) => seed,
        None => bail!("invalid seed length"),
    };
    let (public_key, secret_key) = ed25519::keypair_from_seed(&seed);
    Ok(SignKeyPair{
        public_key,
        secret_key,
    })
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

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ed25519::PublicKey, D::Error>
        where D: Deserializer<'de>
    {
        let b64 = String::deserialize(deserializer)?;
        let bytes = base64::decode(&b64).map_err(serde::de::Error::custom)?;
        Ok(
            ed25519::PublicKey::from_slice(&bytes)
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

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ed25519::SecretKey, D::Error>
        where D: Deserializer<'de>
    {
        let b64 = String::deserialize(deserializer)?;
        let bytes = base64::decode(&b64).map_err(serde::de::Error::custom)?;
        Ok(
            ed25519::SecretKey::from_slice(&bytes)
                .ok_or(serde::de::Error::custom("ed25519::SecretKey::from_slice failed"))?
        )
    }
}

extern crate rand;
use self::rand::Rng;

use super::Result;

pub fn gen_sign_key_pair() -> Result<SignKeyPair> {
    sign_keypair_from_seed(&gen_sign_key_pair_seed()?)
}

pub fn gen_sign_key_pair_seed() -> Result<Vec<u8>> {
    let mut seed = vec![0u8; ed25519::SEEDBYTES];
    let mut rng = rand::os::OsRng::new()?;
    rng.fill_bytes(&mut seed);
    Ok(seed)
}
