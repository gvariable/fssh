use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use std::{fs, path::Path};

pub struct EncryptionManager {
    key: Key<Aes256Gcm>,
}

impl EncryptionManager {
    pub fn new<P: AsRef<Path>>(file: P) -> anyhow::Result<Self> {
        let key = match fs::read(file.as_ref()) {
            Ok(bytes) => Key::<Aes256Gcm>::from_slice(&bytes).clone(),
            Err(_) => {
                let key = Aes256Gcm::generate_key(OsRng);
                fs::write(file, key.as_slice())?;
                key
            }
        };

        Ok(EncryptionManager { key })
    }

    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut encrypted = Vec::new();
        encrypted.extend_from_slice(&nonce);
        encrypted.extend_from_slice(&ciphertext);
        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce = &data[..12];
        let ciphertext = &data[12..];
        let cipher = Aes256Gcm::new(&self.key);
        cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
