use anyhow::{Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let digest = hasher.finalize();
    let mut key = [0_u8; 32];
    key.copy_from_slice(&digest);
    key
}

pub fn sign_payload(secret: &str, payload: &[u8]) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| anyhow!("invalid hmac key: {e}"))?;
    mac.update(payload);
    Ok(STANDARD.encode(mac.finalize().into_bytes()))
}

pub fn verify_payload_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    let Ok(decoded_sig) = STANDARD.decode(signature) else {
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(payload);
    mac.verify_slice(&decoded_sig).is_ok()
}

pub fn encrypt_text(secret: &str, plaintext: &str) -> String {
    let key = derive_key(secret);
    let mut nonce = [0_u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce);

    let mut out = Vec::with_capacity(16 + plaintext.len());
    out.extend_from_slice(&nonce);
    for (i, byte) in plaintext.as_bytes().iter().enumerate() {
        let k = key[i % key.len()] ^ nonce[i % nonce.len()];
        out.push(byte ^ k);
    }
    STANDARD.encode(out)
}

pub fn decrypt_text(secret: &str, ciphertext: &str) -> Result<String> {
    let decoded = STANDARD
        .decode(ciphertext)
        .map_err(|e| anyhow!("invalid base64 ciphertext: {e}"))?;
    if decoded.len() < 16 {
        return Err(anyhow!("ciphertext too short"));
    }

    let nonce = &decoded[..16];
    let encrypted = &decoded[16..];
    let key = derive_key(secret);

    let mut plain = Vec::with_capacity(encrypted.len());
    for (i, byte) in encrypted.iter().enumerate() {
        let k = key[i % key.len()] ^ nonce[i % nonce.len()];
        plain.push(byte ^ k);
    }

    String::from_utf8(plain).map_err(|e| anyhow!("invalid utf8 plaintext: {e}"))
}

#[cfg(test)]
mod tests {
    use super::{decrypt_text, encrypt_text, sign_payload, verify_payload_signature};

    #[test]
    fn sign_and_verify_payload() {
        let secret = "top-secret";
        let payload = br#"{"a":1}"#;
        let signature = sign_payload(secret, payload).expect("signature should be generated");
        assert!(verify_payload_signature(secret, payload, &signature));
        assert!(!verify_payload_signature("wrong", payload, &signature));
    }

    #[test]
    fn encrypt_and_decrypt_roundtrip() {
        let secret = "queue-key";
        let plaintext = "hello world";
        let encrypted = encrypt_text(secret, plaintext);
        let decrypted = decrypt_text(secret, &encrypted).expect("decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }
}
