use super::sodiumoxide::crypto::secretbox;
use super::sodiumoxide::init;

pub fn gen() -> Vec<u8> {
    secretbox::gen_key().0.to_vec()
}

pub fn encrypt(plaintext: &[u8], symmetric_key: &[u8]) -> super::Result<Vec<u8>> {
    init()?;
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(
        plaintext, 
        &nonce, 
        &secretbox::Key::from_slice(symmetric_key).ok_or("invalid symmetric key")?,
        );

    let nonce_and_ciphertext : Vec<u8> = [nonce.0.as_ref().into(), ciphertext.as_slice()].concat();
    Ok(nonce_and_ciphertext)
}

pub fn decrypt(symmetric_key: Vec<u8>, nonce_and_ciphertext: Vec<u8>) -> super::Result<Vec<u8>> {
    if nonce_and_ciphertext.len() < secretbox::NONCEBYTES {
        bail!("ciphertext too short")
    }
    let checked_nonce = secretbox::Nonce::from_slice(&nonce_and_ciphertext[0..secretbox::NONCEBYTES])
        .ok_or("invalid nonce")?;
    let checked_key = secretbox::Key::from_slice(&symmetric_key).ok_or("invalid key")?;

    Ok(secretbox::open(&nonce_and_ciphertext[secretbox::NONCEBYTES..], &checked_nonce, &checked_key)
        .map_err(|_| "secretbox::open failed")?)
}

pub struct EphemeralEncryption {
    pub symmetric_key: Vec<u8>,
    pub nonce_and_ciphertext: Vec<u8>,
}

pub fn ephemeral_encrypt(plaintext: Vec<u8>) -> super::Result<EphemeralEncryption> {
    init()?;
    let key = secretbox::gen_key();
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(&plaintext, &nonce, &key);

    let nonce_and_ciphertext : Vec<u8> = [nonce.0.as_ref().into(), ciphertext.as_slice()].concat();
    Ok(EphemeralEncryption{
        symmetric_key: key.0.as_ref().into(),
        nonce_and_ciphertext,
    })
}
