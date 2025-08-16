/**
 * Cryptography helpers using Web Crypto API.
 * Implements AES-GCM with a key derived via PBKDF2(password),
 * matching the server's symmetric scheme.
 */
import { arrayBufferToBase64, base64ToArrayBuffer } from "./utils.js"

/**
 * Derive an AES-GCM CryptoKey from a password and salt using PBKDF2.
 * @param {string} password - The password to derive the key from
 * @param {string} [saltB64] - Optional base64-encoded salt. If not provided, generates a random salt.
 * @returns {Promise<{ key: CryptoKey, salt: string }>} The derived key and salt (base64)
 */
export async function deriveKeyFromPasswordWithSalt(password, saltB64) {
  const encoder = new TextEncoder()
  const passwordData = encoder.encode(password)

  let salt
  if (saltB64) {
    // Use provided salt
    salt = base64ToArrayBuffer(saltB64)
  } else {
    // Generate a new random 16-byte salt
    salt = crypto.getRandomValues(new Uint8Array(16))
  }

  // Import password as key material
  const keyMaterial = await crypto.subtle.importKey("raw", passwordData, { name: "PBKDF2" }, false, ["deriveKey"])

  // Derive key using PBKDF2
  const key = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: salt,
      iterations: 100000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  )

  return {
    key: key,
    salt: arrayBufferToBase64(salt),
  }
}

/**
 * Derive an AES-GCM CryptoKey from a plain-text password using PBKDF2.
 * Uses a fixed salt for backward compatibility with existing systems.
 * For new implementations, consider using deriveKeyFromPasswordWithSalt for better security.
 * @param {string} password
 * @returns {Promise<CryptoKey>}
 */
export async function deriveKeyFromPassword(password) {
  const encoder = new TextEncoder()
  const passwordData = encoder.encode(password)

  // Use a fixed salt for backward compatibility
  const salt = new Uint8Array(16)
  salt.fill(0x42) // Fixed salt for legacy compatibility

  // Import password as key material
  const keyMaterial = await crypto.subtle.importKey("raw", passwordData, { name: "PBKDF2" }, false, ["deriveKey"])

  // Derive key using PBKDF2
  const key = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: salt,
      iterations: 100000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  )

  return key
}

/**
 * Encrypt a UTF-8 string with AES-GCM using a password (with secure salt).
 * @param {string} password - Password to derive encryption key from
 * @param {string} plaintext - Message to encrypt
 * @returns {Promise<{ encrypted_message: string, nonce: string, salt: string }>} Base64 values
 */
export async function encryptWithPassword(password, plaintext) {
  // Generate a new salt for this encryption
  const { key, salt } = await deriveKeyFromPasswordWithSalt(password)

  const nonce = crypto.getRandomValues(new Uint8Array(12))
  const encoder = new TextEncoder()
  const plaintextBuffer = encoder.encode(plaintext)

  const encrypted = await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce }, key, plaintextBuffer)

  return {
    encrypted_message: arrayBufferToBase64(encrypted),
    nonce: arrayBufferToBase64(nonce),
    salt: salt,
  }
}

/**
 * Decrypt a Base64 AES-GCM payload using a password and salt.
 * @param {string} password - Password to derive decryption key from
 * @param {string} encryptedB64 - Base64 ciphertext (with auth tag)
 * @param {string} nonceB64 - Base64 12-byte IV
 * @param {string} saltB64 - Base64 16-byte salt used for key derivation
 * @returns {Promise<string>} Decrypted UTF-8 text
 */
export async function decryptWithPassword(password, encryptedB64, nonceB64, saltB64) {
  // Derive the key using the provided salt
  const { key } = await deriveKeyFromPasswordWithSalt(password, saltB64)

  const encrypted = base64ToArrayBuffer(encryptedB64)
  const nonce = base64ToArrayBuffer(nonceB64)

  const decrypted = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce }, key, encrypted)

  const decoder = new TextDecoder()
  return decoder.decode(decrypted)
}

/**
 * Encrypt a UTF-8 string with AES-GCM using the provided key.
 * @param {CryptoKey} key - AES-GCM key
 * @param {string} plaintext - Message to encrypt
 * @returns {Promise<{ encrypted_message: string, nonce: string }>} Base64 values
 */
export async function encryptWithKey(key, plaintext) {
  const nonce = crypto.getRandomValues(new Uint8Array(12))
  const encoder = new TextEncoder()
  const plaintextBuffer = encoder.encode(plaintext)
  const encrypted = await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce }, key, plaintextBuffer)
  return {
    encrypted_message: arrayBufferToBase64(encrypted),
    nonce: arrayBufferToBase64(nonce),
  }
}

/**
 * Decrypt a Base64 AES-GCM payload using the provided key.
 * @param {CryptoKey} key - AES-GCM key
 * @param {string} encryptedB64 - Base64 ciphertext (with auth tag)
 * @param {string} nonceB64 - Base64 12-byte IV
 * @returns {Promise<string>} Decrypted UTF-8 text
 */
export async function decryptWithKey(key, encryptedB64, nonceB64) {
  const encrypted = base64ToArrayBuffer(encryptedB64)
  const nonce = base64ToArrayBuffer(nonceB64)
  const decrypted = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce }, key, encrypted)
  const decoder = new TextDecoder()
  return decoder.decode(decrypted)
}
