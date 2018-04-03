use errors::Result;

extern crate sodiumoxide;

pub mod sign;
pub use self::sign::*;

pub mod secretbox;

pub mod box_;
pub use self::box_::*;

extern crate rand;
use self::rand::Rng;

pub fn random_nonce() -> Result<Vec<u8>> {
    let mut nonce = [0u8; 32].to_vec();
    rand::os::OsRng::new()?.fill_bytes(&mut nonce);
    Ok(nonce)
}
