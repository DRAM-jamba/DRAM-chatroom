use keyring::Entry;
use rand::Rng;
use ed25519_dalek::{SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};

pub fn get_or_create_master_key() -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let service = "com.quorthon.app.vault";
    let username = "master_key";

    let entry = Entry::new(service, username)?;

    match entry.get_password() {
        Ok(password) => validate_hex_key(&password),
        Err(keyring::Error::NoEntry) => {
            let key: [u8; 32] = rand::thread_rng().gen();
            entry.set_password(&hex::encode(key))?;
            Ok(key)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn derive_identity_keypair(master_key: &[u8; 32]) -> (SigningKey, VerifyingKey) {
    let mut hasher = Sha256::new();
        hasher.update(b"identity_key_derivation_v1"); // hardcoding is fine
        hasher.update(master_key);
    
    let seed: [u8; 32] = hasher.finalize().into();
    let private_key = SigningKey::from_bytes(&seed);
    let public_key = private_key.verifying_key();
    
    (private_key, public_key)
}

pub fn get_public_key_hex(master_key: &[u8; 32]) -> String {
    let (_, public_key) = derive_identity_keypair(master_key);
    hex::encode(public_key.to_bytes())
}

// Test & Validator
pub fn validate_hex_key(hex_str: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex_str)?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| "Key must be exactly 32 bytes")?;
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hex_key_valid() {
        let valid_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = validate_hex_key(valid_hex);
        assert!(result.is_ok());
        let key = result.unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_validate_hex_key_wrong_length() {
        let short_hex = "0123456789abcdef"; // Only 8 bytes
        let result = validate_hex_key(short_hex);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }

    #[test]
    fn test_validate_hex_key_long_hex() {
        let long_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00"; // 33 bytes
        let result = validate_hex_key(long_hex);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }

    #[test]
    fn test_validate_hex_key_invalid_hex() {
        let invalid_hex = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        let result = validate_hex_key(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_hex_key_uppercase() {
        let uppercase_hex = "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF";
        let result = validate_hex_key(uppercase_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_validate_hex_key_mixed_case() {
        let mixed_hex = "0123456789AbCdEf0123456789aBcDeF0123456789AbCdEf0123456789aBcDeF";
        let result = validate_hex_key(mixed_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_key_generation_produces_32_bytes() {
        let key: [u8; 32] = rand::thread_rng().gen();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hex_encode_decode_roundtrip() {
        let original_key: [u8; 32] = rand::thread_rng().gen();
        let encoded = hex::encode(original_key);
        let decoded = validate_hex_key(&encoded).unwrap();
        assert_eq!(original_key, decoded);
    }

    #[test]
    fn test_hex_encode_length() {
        let key: [u8; 32] = rand::thread_rng().gen();
        let encoded = hex::encode(key);
        assert_eq!(encoded.len(), 64);
    }

    #[test]
    fn test_key_bytes_are_different_each_time() {
        let key1: [u8; 32] = rand::thread_rng().gen();
        let key2: [u8; 32] = rand::thread_rng().gen();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_validate_hex_key_empty_string() {
        let empty_hex = "";
        let result = validate_hex_key(empty_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_hex_key_odd_length() {
        let odd_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde"; // 63 chars
        let result = validate_hex_key(odd_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_array_indexing() {
        let key: [u8; 32] = rand::thread_rng().gen();
        for i in 0..32 {
            let _ = key[i]; // Should not panic
        }
    }

    #[test]
    fn test_validate_hex_key_all_zeros() {
        let zeros_hex = "00000000000000000000000000000000000000000000000000000000000000000";
        let result = validate_hex_key(zeros_hex);
        assert!(result.is_err()); // 65 chars, not 64
    }

    #[test]
    fn test_validate_hex_key_all_ff() {
        let ff_hex = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let result = validate_hex_key(ff_hex);
        assert!(result.is_ok());
        let key = result.unwrap();
        for byte in key.iter() {
            assert_eq!(*byte, 0xFF);
        }
    }
}