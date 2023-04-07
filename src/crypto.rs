use crate::app::data::{Error, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    aes::cipher::InvalidLength,
    Aes256Gcm, Nonce,
};
use rand::{thread_rng, Rng};

const NONCE_SIZE: usize = 12;

impl From<InvalidLength> for Error {
    fn from(_: InvalidLength) -> Self {
        Error::from("incorrect key size")
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(value: aes_gcm::Error) -> Self {
        Error::from(value.to_string())
    }
}

fn get_cipher(key: &str) -> Result<Aes256Gcm> {
    let key = key.as_bytes().to_vec();
    let mut fixed_key: Vec<u8> = vec![0; 32];
    fixed_key.splice(0..key.len(), key);
    let cipher = Aes256Gcm::new_from_slice(fixed_key.as_slice())?;
    Ok(cipher)
}

pub fn encrypt(plaintext: &Vec<u8>, key: &str) -> Result<Vec<u8>> {
    let cipher = get_cipher(key)?;
    let nonce_data: [u8; NONCE_SIZE] = thread_rng().gen();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_data), plaintext.as_slice())
        .map_err(|e| Error::from(format!("encryption failure [{e}]")))?;
    ciphertext.extend_from_slice(&nonce_data);
    Ok(ciphertext)
}

pub fn decrypt(ciphertext: &Vec<u8>, key: &str) -> Result<Vec<u8>> {
    let cipher = get_cipher(key)?;
    let split_at = ciphertext.len().saturating_sub(NONCE_SIZE);
    (split_at > 0)
        .then_some(())
        .ok_or(Error::from("file too small to decrypt"))?;
    let (ciphertext, nonce_data) = ciphertext.split_at(split_at);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_data), ciphertext)
        .map_err(|e| Error::from(format!("decryption failure [{e}]")))?;
    Ok(plaintext)
}
