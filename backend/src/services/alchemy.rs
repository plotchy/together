use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use alloy::primitives::U256;
use crate::constants::*;

// Rate limiting structure
#[derive(Debug)]
struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests_per_minute: u32,
}

impl RateLimiter {
    fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests_per_minute,
        }
    }

    async fn can_make_request(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(RATE_LIMIT_WINDOW_SECONDS);
        
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests
        requests.retain(|&time| time > minute_ago);
        
        if requests.len() < self.max_requests_per_minute as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlchemyService {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl AlchemyService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://worldchain-mainnet.g.alchemy.com/nft/v3".to_string(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(ALCHEMY_RATE_LIMIT_PER_MINUTE))),
        }
    }

}

fn uint256_to_bytes20_hex(token_id_str: &str) -> Result<String> {
    // Parse the token ID as a U256 from decimal string
    let token_id_u256 = U256::from_str_radix(token_id_str, 10)
        .map_err(|_| anyhow::anyhow!("Invalid token ID: {}", token_id_str))?;
    
    // Convert to full 64-char hex (32 bytes, padded with zeros)
    let full_hex = format!("{:064x}", token_id_u256);
    
    // Take the rightmost 40 chars (20 bytes) and add 0x prefix
    let bytes20_hex = &full_hex[full_hex.len() - 40..];
    Ok(format!("0x{}", bytes20_hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint256_to_bytes20_hex() {
        
        // Test with a large number that would overflow u128
        let result = uint256_to_bytes20_hex("226290979940146177001751347620814275862989157").unwrap();
        assert!(result.starts_with("0x"));
        assert_eq!(result.len(), 42); // 0x + 40 hex chars = 42 total (20 bytes)
        println!("result: {}", result);
    }
}
