// src/core/api_key.rs — Opaque API key type with HMAC-SHA256 hashing and timing-safe comparison

use base64::Engine;
use ring::hmac;
use std::fmt;

const HMAC_PREFIX: &str = "$hmac-sha256$";
const HMAC_KEY_VALUE: &[u8] = b"rush-sync-api-key-v1";

#[derive(Clone)]
enum ApiKeySource {
    Empty,
    Toml(String),
    EnvVar(String),
}

#[derive(Clone)]
pub struct ApiKey {
    source: ApiKeySource,
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

impl ApiKey {
    pub fn empty() -> Self {
        Self {
            source: ApiKeySource::Empty,
        }
    }

    /// Parse a value from the TOML config file.
    /// Recognises `$hmac-sha256$...` as a pre-hashed key, otherwise stores as plaintext.
    pub fn from_toml(value: &str) -> Self {
        if value.is_empty() {
            return Self::empty();
        }
        Self {
            source: ApiKeySource::Toml(value.to_string()),
        }
    }

    /// Value comes from an environment variable — will never be written back to TOML.
    pub fn from_env(value: &str) -> Self {
        if value.is_empty() {
            return Self::empty();
        }
        Self {
            source: ApiKeySource::EnvVar(value.to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.source, ApiKeySource::Empty)
    }

    /// Timing-safe verification of a provided plaintext key.
    pub fn verify(&self, provided: &str) -> bool {
        match &self.source {
            ApiKeySource::Empty => false,
            ApiKeySource::Toml(stored) | ApiKeySource::EnvVar(stored) => {
                if let Some(hash_b64) = stored.strip_prefix(HMAC_PREFIX) {
                    // Stored value is a hash — compute HMAC of the provided key and compare
                    verify_hmac_hash(hash_b64, provided)
                } else {
                    // Stored value is plaintext — timing-safe via HMAC:
                    // sign(stored) then verify(provided) against that tag
                    let key = hmac::Key::new(hmac::HMAC_SHA256, HMAC_KEY_VALUE);
                    let tag = hmac::sign(&key, stored.as_bytes());
                    hmac::verify(&key, provided.as_bytes(), tag.as_ref()).is_ok()
                }
            }
        }
    }

    /// Value to persist in TOML. Returns `""` for env-var sourced keys.
    pub fn to_toml_value(&self) -> String {
        match &self.source {
            ApiKeySource::Empty => String::new(),
            ApiKeySource::Toml(v) => v.clone(),
            ApiKeySource::EnvVar(_) => String::new(),
        }
    }
}

/// Compute `$hmac-sha256$<base64>` for a plaintext API key.
pub fn hash_api_key(plaintext: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, HMAC_KEY_VALUE);
    let tag = hmac::sign(&key, plaintext.as_bytes());
    let b64 = base64::engine::general_purpose::STANDARD.encode(tag.as_ref());
    format!("{}{}", HMAC_PREFIX, b64)
}

fn verify_hmac_hash(hash_b64: &str, provided: &str) -> bool {
    let Ok(expected_tag) = base64::engine::general_purpose::STANDARD.decode(hash_b64) else {
        return false;
    };
    let key = hmac::Key::new(hmac::HMAC_SHA256, HMAC_KEY_VALUE);
    hmac::verify(&key, provided.as_bytes(), &expected_tag).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_key_never_verifies() {
        let key = ApiKey::empty();
        assert!(key.is_empty());
        assert!(!key.verify("anything"));
    }

    #[test]
    fn test_plaintext_match() {
        let key = ApiKey::from_toml("my-secret-key");
        assert!(key.verify("my-secret-key"));
    }

    #[test]
    fn test_plaintext_mismatch() {
        let key = ApiKey::from_toml("my-secret-key");
        assert!(!key.verify("wrong-key"));
    }

    #[test]
    fn test_hash_match() {
        let hashed = hash_api_key("super-secret");
        let key = ApiKey::from_toml(&hashed);
        assert!(key.verify("super-secret"));
    }

    #[test]
    fn test_hash_mismatch() {
        let hashed = hash_api_key("super-secret");
        let key = ApiKey::from_toml(&hashed);
        assert!(!key.verify("wrong-secret"));
    }

    #[test]
    fn test_env_to_toml_value_is_empty() {
        let key = ApiKey::from_env("env-secret");
        assert!(key.verify("env-secret"));
        assert_eq!(key.to_toml_value(), "");
    }

    #[test]
    fn test_toml_to_toml_value_roundtrip() {
        let key = ApiKey::from_toml("stored-value");
        assert_eq!(key.to_toml_value(), "stored-value");
    }

    #[test]
    fn test_hash_format() {
        let hashed = hash_api_key("test");
        assert!(hashed.starts_with(HMAC_PREFIX));
        // Base64 of 32-byte HMAC-SHA256 tag = 44 chars
        let b64_part = hashed.strip_prefix(HMAC_PREFIX).unwrap();
        assert_eq!(b64_part.len(), 44);
    }
}
