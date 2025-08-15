//! Symmetric encryption helpers for chat payloads.
//! Uses AES-256-GCM and a key derived by PBKDF2 KDF.
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

/// High-level encryptor that encapsulates key derivation and AES-GCM usage.
pub struct EncryptionManager {
    cipher: Aes256Gcm,
}

impl EncryptionManager {
    /// Create a new encryption manager from a plain-text password.
    /// Uses PBKDF2 with a fixed salt for compatibility with client.
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Use a fixed salt for compatibility with client-side implementation
        // In a production environment, this should be randomly generated and stored
        let salt: [u8; 16] = [0x42; 16]; // Fixed salt
        
        // Derive key using PBKDF2
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100000, &mut key);
        
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Failed to create cipher: {e}"))?;

        Ok(Self { cipher })
    }

    /// Encrypt a UTF-8 message, returning (ciphertext_b64, nonce_b64, salt_b64).
    pub fn encrypt_message(
        &self,
        plaintext: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {e}"))?;

        let encrypted_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(nonce);
        // For compatibility, we're not using salt in the return value anymore
        let salt_b64 = general_purpose::STANDARD.encode([0x42; 16]); // Fixed salt

        Ok((encrypted_b64, nonce_b64, salt_b64))
    }
}