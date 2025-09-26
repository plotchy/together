use alloy::primitives::{U256, Address};
use anyhow::Result;

/// Converts a token ID (as string or U256) to a bytes20 hex string for database queries
/// 
/// Process: U256 -> truncate to U160 -> Address (bytes20) -> hex string
/// This follows the pattern: token_id -> cast_hash conversion
pub fn token_id_to_cast_hash(token_id: &str) -> Result<String> {
    // Parse token ID as U256
    let token_id_u256: U256 = token_id.parse()
        .map_err(|e| anyhow::anyhow!("Invalid token ID '{}': {}", token_id, e))?;
    
    token_id_u256_to_cast_hash(token_id_u256)
}

/// Converts a U256 token ID to a bytes20 hex string for database queries
pub fn token_id_u256_to_cast_hash(token_id: U256) -> Result<String> {
    // Convert U256 to bytes and take the last 20 bytes (160 bits)
    let bytes32 = token_id.to_be_bytes::<32>();
    let bytes20 = &bytes32[12..32]; // Take last 20 bytes
    
    // Create Address from bytes20
    let address = Address::from_slice(bytes20);
    
    // Return as lowercase hex string (already includes 0x prefix)
    Ok(address.to_string().to_lowercase())
}

/// Converts a cast hash (bytes20 hex string) back to a U256 token ID
pub fn cast_hash_to_token_id(cast_hash: &str) -> Result<U256> {
    // Parse as Address to get bytes20
    let address: Address = cast_hash.parse()
        .map_err(|e| anyhow::anyhow!("Invalid cast hash '{}': {}", cast_hash, e))?;
    
    // Convert to U256 (zero-padded in the high bits)
    let bytes20 = address.as_slice();
    let mut bytes32 = [0u8; 32];
    bytes32[12..32].copy_from_slice(bytes20); // Put bytes20 in the low bits
    
    Ok(U256::from_be_bytes(bytes32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_id_to_cast_hash() {
        let token_id = "226290979940146177001751347620814275862989157";
        let cast_hash = token_id_to_cast_hash(token_id).unwrap();

        println!("cast_hash: {}", cast_hash);
        // Should be a valid hex string with 0x prefix and 40 hex chars
        assert!(cast_hash.starts_with("0x"));
        assert_eq!(cast_hash.len(), 42);
        
        // Should be able to convert back
        let converted_back = cast_hash_to_token_id(&cast_hash).unwrap();
        assert_eq!(converted_back, U256::from_str_radix(token_id, 10).unwrap());
        println!("converted_back: {}", converted_back);
    }

    #[test]
    fn test_u256_to_cast_hash() {
        let token_id = U256::from(123456789u64);
        let cast_hash = token_id_u256_to_cast_hash(token_id).unwrap();
        
        assert!(cast_hash.starts_with("0x"));
        assert_eq!(cast_hash.len(), 42);
        
        let converted_back = cast_hash_to_token_id(&cast_hash).unwrap();
        assert_eq!(converted_back, token_id);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_token_id = "999999999999999999";
        
        // token_id -> cast_hash -> token_id
        let cast_hash = token_id_to_cast_hash(original_token_id).unwrap();
        let recovered_token_id = cast_hash_to_token_id(&cast_hash).unwrap();
        
        assert_eq!(recovered_token_id, U256::from_str_radix(original_token_id, 10).unwrap());
    }
}
