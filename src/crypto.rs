use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{thread_rng, Rng};

const NONCE_SIZE: usize = 12;

fn get_cipher(key: &str) -> Aes256Gcm {
    let key = key.as_bytes().to_vec();
    let mut fixed_key: Vec<u8> = vec![0; 32];
    fixed_key.splice(0..key.len(), key);
    Aes256Gcm::new_from_slice(fixed_key.as_slice()).expect("failed to generate key")
}

pub fn encrypt(plaintext: &Vec<u8>, key: &str) -> Vec<u8> {
    let cipher = get_cipher(key);
    let nonce_data: [u8; NONCE_SIZE] = thread_rng().gen();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_data), plaintext.as_slice())
        .expect("failed to encrypt");
    ciphertext.extend_from_slice(&nonce_data);
    ciphertext
}

pub fn decrypt(ciphertext: &Vec<u8>, key: &str) -> Vec<u8> {
    let cipher = get_cipher(key);
    let (ciphertext, nonce_data) = ciphertext.split_at(ciphertext.len() - NONCE_SIZE);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_data), ciphertext)
        .expect("failed to decrypt");
    plaintext
}
