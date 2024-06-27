use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use std::{fs, path::Path};

/// An encryption manager that uses AES-256-GCM to encrypt and decrypt data.
pub struct EncryptionManager {
    key: Key<Aes256Gcm>,
}

impl EncryptionManager {
    /// Creates a new [`EncryptionManager`] instance using the encryption key stored in the given file.
    ///
    /// If the file does not exist, a new key is created.
    /// If the file exists, the key is loaded from the file.
    pub fn new<P: AsRef<Path>>(key_path: P) -> anyhow::Result<Self> {
        let key = match fs::read(key_path.as_ref()) {
            Ok(bytes) => Key::<Aes256Gcm>::from_slice(&bytes).clone(),
            Err(_) => {
                let key = Aes256Gcm::generate_key(OsRng);
                fs::write(key_path, key.as_slice())?;
                key
            }
        };

        Ok(EncryptionManager { key })
    }

    /// Encrypts the given data and returns the ciphertext along with the nonce.
    ///
    /// The nonce is generated for each message to ensure uniqueness. The returned
    /// vector contains the nonce followed by the ciphertext.
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

    /// Decrypts the given encrypted text, which consists of a nonce and ciphertext, and returns the original message.
    ///
    /// The input data is expected to contain the nonce as the first 12 bytes, followed by the ciphertext.
    pub fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce = &data[..12];
        let ciphertext = &data[12..];
        let cipher = Aes256Gcm::new(&self.key);
        cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
