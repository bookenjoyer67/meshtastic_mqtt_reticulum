use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use anyhow::{Result, anyhow};
use rand_core::RngCore;

/// Encrypt a message using ChaCha20Poly1305 with a PSK
/// Returns base64 encoded ciphertext
pub fn encrypt_message(psk: &str, plaintext: &str) -> Result<String> {
    // Decode the base64 PSK
    let psk_bytes = BASE64.decode(psk)?;
    
    // Ensure PSK is the right length for ChaCha20Poly1305 (32 bytes)
    let mut key_bytes = [0u8; 32];
    if psk_bytes.len() >= 32 {
        key_bytes.copy_from_slice(&psk_bytes[..32]);
    } else {
        // Pad with zeros if PSK is too short (not ideal, but better than nothing)
        // In production, you'd want to use a KDF like Argon2 or PBKDF2
        psk_bytes.iter().enumerate().for_each(|(i, &b)| key_bytes[i] = b);
    }
    
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    
    // Generate a random nonce
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    
    // Encrypt the message
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;
    
    // Combine nonce and ciphertext, then base64 encode
    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    
    Ok(BASE64.encode(&combined))
}

/// Decrypt a message using ChaCha20Poly1305 with a PSK
/// Takes base64 encoded ciphertext (nonce + ciphertext)
pub fn decrypt_message(psk: &str, ciphertext_b64: &str) -> Result<String> {
    // Decode the base64 ciphertext
    let combined = BASE64.decode(ciphertext_b64)?;
    
    // Ensure we have enough data (nonce is 12 bytes for ChaCha20Poly1305)
    if combined.len() < 12 {
        return Err(anyhow!("Ciphertext too short"));
    }
    
    // Split nonce and ciphertext
    let nonce = &combined[..12];
    let ciphertext = &combined[12..];
    
    // Decode the base64 PSK
    let psk_bytes = BASE64.decode(psk)?;
    
    // Ensure PSK is the right length for ChaCha20Poly1305 (32 bytes)
    let mut key_bytes = [0u8; 32];
    if psk_bytes.len() >= 32 {
        key_bytes.copy_from_slice(&psk_bytes[..32]);
    } else {
        // Pad with zeros if PSK is too short
        psk_bytes.iter().enumerate().for_each(|(i, &b)| key_bytes[i] = b);
    }
    
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    // Decrypt the message
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;
    
    String::from_utf8(plaintext_bytes).map_err(|e| anyhow!("Invalid UTF-8 after decryption: {}", e))
}

/// Generate a random PSK (32 bytes, base64 encoded)
pub fn generate_random_psk() -> String {
    let mut rng = OsRng;
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    BASE64.encode(key_bytes)
}

/// Check if a PSK is strong enough
pub fn is_strong_psk(psk: &str) -> bool {
    if let Ok(decoded) = BASE64.decode(psk) {
        decoded.len() >= 32  // At least 32 bytes (256 bits)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let psk = generate_random_psk();
        let plaintext = "Hello, encrypted world!";
        
        let ciphertext = encrypt_message(&psk, plaintext).unwrap();
        let decrypted = decrypt_message(&psk, &ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_decryption_with_short_psk() {
        // Test with a short PSK (should be padded)
        let psk = BASE64.encode("short");
        let plaintext = "Test message";
        
        let ciphertext = encrypt_message(&psk, plaintext).unwrap();
        let decrypted = decrypt_message(&psk, &ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_psks_dont_decrypt() {
        let psk1 = generate_random_psk();
        let psk2 = generate_random_psk();
        let plaintext = "Secret message";
        
        let ciphertext = encrypt_message(&psk1, plaintext).unwrap();
        
        // Should fail to decrypt with wrong PSK
        assert!(decrypt_message(&psk2, &ciphertext).is_err());
    }

    #[test]
    fn test_is_strong_psk() {
        let weak_psk = BASE64.encode("short");
        let strong_psk = generate_random_psk();
        
        assert!(!is_strong_psk(&weak_psk));
        assert!(is_strong_psk(&strong_psk));
    }
}