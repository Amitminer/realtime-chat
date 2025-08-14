//! Symmetric encryption helpers for chat payloads.
//! Uses AES-256-GCM and a key derived by SHA-256(password).
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};

/// High-level encryptor that encapsulates key derivation and AES-GCM usage.
pub struct EncryptionManager {
    cipher: Aes256Gcm,
}

impl EncryptionManager {
    /// Create a new encryption manager from a plain-text password.
    pub fn new(password: &str) -> Self {
        // Derive key from password using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key_bytes = hasher.finalize();

        let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("Failed to create cipher");
        Self { cipher }
    }

    /// Encrypt a UTF-8 message, returning (ciphertext_b64, nonce_b64).
    pub fn encrypt_message(
        &self,
        plaintext: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {e}"))?;

        let encrypted_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(nonce);
        Ok((encrypted_b64, nonce_b64))
    }
}
