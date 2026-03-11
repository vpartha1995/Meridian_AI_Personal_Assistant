use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use zeroize::Zeroizing;

/// Encrypts `plaintext` with AES-256-GCM.
/// Returns a base64-encoded string: `<12-byte nonce>||<ciphertext+tag>`.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<String> {
    let key   = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce  = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, plaintext)
        .context("AES-GCM encryption failed")?;

    let mut blob = nonce.to_vec();
    blob.extend(ct);
    Ok(B64.encode(blob))
}

/// Decrypts a base64-encoded blob produced by `encrypt`.
pub fn decrypt(encoded: &str, key: &[u8; 32]) -> Result<Zeroizing<Vec<u8>>> {
    let blob = B64.decode(encoded).context("Base64 decode failed")?;
    anyhow::ensure!(blob.len() > 12, "Ciphertext too short");

    let (nonce_bytes, ct) = blob.split_at(12);
    let nonce  = Nonce::from_slice(nonce_bytes);
    let key    = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let plain = cipher
        .decrypt(nonce, ct)
        .context("AES-GCM decryption failed — wrong key or corrupted data")?;

    Ok(Zeroizing::new(plain))
}

/// Encrypt a UTF-8 string and return base64.
pub fn encrypt_str(s: &str, key: &[u8; 32]) -> Result<String> {
    encrypt(s.as_bytes(), key)
}

/// Decrypt and return UTF-8 string.
pub fn decrypt_str(encoded: &str, key: &[u8; 32]) -> Result<String> {
    let bytes = decrypt(encoded, key)?;
    String::from_utf8(bytes.to_vec()).context("Decrypted bytes are not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [42u8; 32];
        let msg = "Hello, Meridian!";
        let enc = encrypt_str(msg, &key).unwrap();
        let dec = decrypt_str(&enc, &key).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn different_nonces_per_call() {
        let key = [1u8; 32];
        let a = encrypt_str("same", &key).unwrap();
        let b = encrypt_str("same", &key).unwrap();
        assert_ne!(a, b); // random nonces → different ciphertexts
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let enc  = encrypt_str("secret", &key1).unwrap();
        assert!(decrypt_str(&enc, &key2).is_err());
    }
}
