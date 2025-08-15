- backend/src/encryption/mod.rs around lines 18 to 25: the code currently derives
an AES key by directly SHA-256 hashing the password (no salt, weak against
brute-force); replace this with a proper memory-hard KDF (Argon2id) that
generates/accepts a cryptographic salt (use rand/OsRng to create a SaltString),
derive sufficient key material and truncate/expand to 32 bytes for AES-256, and
then create the Aes256Gcm from that key; ensure the constructor returns a Result
to propagate errors, persist/transmit the salt alongside nonce/ciphertext (or
accept an optional salt parameter) so decryption can re-derive the same key, and
add the argon2 and rand dependencies in Cargo.toml.
