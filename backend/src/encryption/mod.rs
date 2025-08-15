//! Symmetric encryption helpers for chat payloads.
//!
//! This module provides encryption and decryption functionality using
//! AES-256-GCM with a key derived from a password using PBKDF2.
//!
//! Features:
//! - AES-256-GCM encryption for message confidentiality and integrity
//! - PBKDF2 key derivation for password-based encryption
//! - Base64 encoding for easy transmission over text-based protocols

use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit, OsRng, rand_core::RngCore},
};
use base64::{Engine as _, engine::general_purpose};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// High-level encryptor that encapsulates key derivation and AES-GCM usage.
///
/// This struct manages the encryption process, including key derivation
/// from a password and AES-256-GCM encryption operations.
pub struct EncryptionManager {
    cipher: Aes256Gcm,
    salt: [u8; 16],
}

impl EncryptionManager {
    /// Create a new encryption manager from a plain-text password.
    ///
    /// This function derives a 256-bit key from the provided password using
    /// PBKDF2 with a random salt. The salt is stored with the encryption manager
    /// and used for all encryption operations.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to derive the encryption key from
    ///
    /// # Returns
    ///
    /// A Result containing the EncryptionManager if successful, or an error
    pub fn new(password: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Generate a random salt for key derivation
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        // Derive key using PBKDF2 with the random salt
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 100000, &mut key);

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
            Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "Failed to create cipher: {e}"
            ))
        })?;

        Ok(Self { cipher, salt })
    }

    /// Encrypt a UTF-8 message, returning (ciphertext_b64, nonce_b64, salt_b64).
    ///
    /// This function encrypts a plaintext message using AES-256-GCM, generating
    /// a random nonce for each encryption operation. The result is returned as
    /// base64-encoded strings for easy transmission.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The UTF-8 string to encrypt
    ///
    /// # Returns
    ///
    /// A Result containing a tuple of (ciphertext, nonce, salt) as base64-encoded
    /// strings if successful, or an error
    pub fn encrypt_message(
        &self,
        plaintext: &str,
    ) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| {
                Box::<dyn std::error::Error + Send + Sync>::from(format!("Encryption failed: {e}"))
            })?;

        let encrypted_b64 = general_purpose::STANDARD.encode(&ciphertext);
        let nonce_b64 = general_purpose::STANDARD.encode(nonce);
        let salt_b64 = general_purpose::STANDARD.encode(self.salt); // Use the stored random salt

        Ok((encrypted_b64, nonce_b64, salt_b64))
    }
}
