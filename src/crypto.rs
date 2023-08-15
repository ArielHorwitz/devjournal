use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use rand::{thread_rng, Rng};

const NONCE_SIZE: usize = 12;

fn get_cipher(key: &str) -> Result<Aes256Gcm> {
    let key = key.as_bytes().to_vec();
    let mut fixed_key: Vec<u8> = vec![0; 32];
    fixed_key.splice(0..key.len(), key);
    let cipher = Aes256Gcm::new_from_slice(fixed_key.as_slice()).map_err(|e| anyhow!(e))?;
    Ok(cipher)
}

pub fn encrypt(plaintext: &[u8], key: &str) -> Result<Vec<u8>> {
    let cipher = get_cipher(key)?;
    let nonce_data: [u8; NONCE_SIZE] = thread_rng().gen();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_data), plaintext)
        .map_err(|e| anyhow!("encryption failure: {e}"))?;
    ciphertext.extend_from_slice(&nonce_data);
    Ok(ciphertext)
}

pub fn decrypt(ciphertext: &[u8], key: &str) -> Result<Vec<u8>> {
    let cipher = get_cipher(key)?;
    let split_at = ciphertext.len().saturating_sub(NONCE_SIZE);
    (split_at > 0)
        .then_some(())
        .ok_or(anyhow!("corrupted file [too small]"))?;
    let (ciphertext, nonce_data) = ciphertext.split_at(split_at);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_data), ciphertext)
        .map_err(|e| anyhow!("decryption failure [{e}]"))?;
    Ok(plaintext)
}
