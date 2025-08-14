/**
 * Cryptography helpers using Web Crypto API.
 * Implements AES-GCM with a key derived via SHA-256(password),
 * matching the server's symmetric scheme.
 */

import { arrayBufferToBase64, base64ToArrayBuffer } from "./utils.js";

/**
 * Derive an AES-GCM CryptoKey from a plain-text password using SHA-256.
 * @param {string} password
 * @returns {Promise<CryptoKey>}
 */
export async function deriveKeyFromPassword(password) {
  const encoder = new TextEncoder();
  const passwordData = encoder.encode(password);
  const hashBuffer = await crypto.subtle.digest("SHA-256", passwordData);

  return crypto.subtle.importKey(
    "raw",
    hashBuffer,
    { name: "AES-GCM" },
    false,
    ["encrypt", "decrypt"]
  );
}

/**
 * Encrypt a UTF-8 string with AES-GCM using the provided key.
 * @param {CryptoKey} key - AES-GCM key
 * @param {string} plaintext - Message to encrypt
 * @returns {Promise<{ encrypted_message: string, nonce: string }>} Base64 values
 */
export async function encryptWithKey(key, plaintext) {
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const encoder = new TextEncoder();
  const plaintextBuffer = encoder.encode(plaintext);

  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: nonce },
    key,
    plaintextBuffer
  );

  return {
    encrypted_message: arrayBufferToBase64(encrypted),
    nonce: arrayBufferToBase64(nonce),
  };
}

/**
 * Decrypt a Base64 AES-GCM payload using the provided key.
 * @param {CryptoKey} key - AES-GCM key
 * @param {string} encryptedB64 - Base64 ciphertext (with auth tag)
 * @param {string} nonceB64 - Base64 12-byte IV
 * @returns {Promise<string>} Decrypted UTF-8 text
 */
export async function decryptWithKey(key, encryptedB64, nonceB64) {
  const encrypted = base64ToArrayBuffer(encryptedB64);
  const nonce = base64ToArrayBuffer(nonceB64);

  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: nonce },
    key,
    encrypted
  );

  const decoder = new TextDecoder();
  return decoder.decode(decrypted);
}
