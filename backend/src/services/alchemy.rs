use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use alloy::primitives::U256;
use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemyNft {
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemyNftsResponse {
    #[serde(rename = "ownedNfts")]
    pub owned_nfts: Vec<AlchemyNft>,
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    #[serde(rename = "pageKey")]
    pub page_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTokenPair {
    pub contract_address: String,
    pub token_id_decimal: String,
    pub token_id_hex: String,
    pub balance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>, // For filtering dwr.eth casts
}

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
            base_url: "https://base-mainnet.g.alchemy.com/nft/v3".to_string(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(ALCHEMY_RATE_LIMIT_PER_MINUTE))),
        }
    }

    pub async fn get_nfts_for_owner(
        &self,
        owner_address: &str,
        contract_addresses: &[&str],
        limit: Option<u32>,
    ) -> Result<Vec<ContractTokenPair>> {
        // Rate limiting
        let mut rate_limiter = self.rate_limiter.lock().await;
        if !rate_limiter.can_make_request(owner_address).await {
            return Err(anyhow::anyhow!("Rate limit exceeded for address: {}", owner_address));
        }
        drop(rate_limiter);

        let mut all_nfts = Vec::new();
        let mut page_key: Option<String> = None;
        let target_limit = limit.unwrap_or(ALCHEMY_DEFAULT_PAGE_SIZE);

        loop {
            let mut url = format!(
                "{}/{}/getNFTsForOwner?owner={}&pageSize={}&withMetadata=false",
                self.base_url, self.api_key, owner_address, ALCHEMY_DEFAULT_PAGE_SIZE
            );

            // Add contract addresses to filter
            for contract_addr in contract_addresses {
                url.push_str(&format!("&contractAddresses%5B%5D={}", contract_addr));
            }

            if let Some(ref key) = page_key {
                url.push_str(&format!("&pageKey={}", key));
            }

            let response = self.client
                .get(&url)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Alchemy API error: {}", error_text));
            }

            let response_data: AlchemyNftsResponse = response.json().await?;
            
            // Convert NFTs to our format
            for nft in response_data.owned_nfts {
                let token_id_hex = uint256_to_bytes20_hex(&nft.token_id)?;
                all_nfts.push(ContractTokenPair {
                    contract_address: nft.contract_address,
                    token_id_decimal: nft.token_id,
                    token_id_hex,
                    balance: nft.balance,
                    image_url: None,
                    metadata_url: None,
                    author: None,
                });

                if all_nfts.len() >= target_limit as usize {
                    return Ok(all_nfts);
                }
            }

            // Check if there's a next page
            page_key = response_data.page_key;
            if page_key.is_none() {
                break;
            }
        }

        Ok(all_nfts)
    }

    pub async fn check_farcaster_pro_og_ownership(&self, owner_address: &str) -> Result<bool> {
        let nfts = self.get_nfts_for_owner(
            owner_address,
            &[FARCASTER_PRO_OG_CONTRACT_ADDRESS],
            Some(1) // Just need to check if they own at least one
        ).await?;

        Ok(!nfts.is_empty())
    }

    pub async fn get_dwrcasts_nfts(&self, owner_address: &str) -> Result<Vec<ContractTokenPair>> {
        self.get_nfts_for_owner(
            owner_address,
            &[DWR_CASTS_CONTRACT_ADDRESS],
            Some(MAX_DWRCASTS_PER_REQUEST)
        ).await
    }

    pub async fn get_farcaster_collectible_casts(&self, owner_address: &str) -> Result<Vec<ContractTokenPair>> {
        self.get_nfts_for_owner(
            owner_address,
            &[FARCASTER_COLLECTIBLE_CASTS_CONTRACT_ADDRESS],
            Some(MAX_FARCASTER_COLLECTIBLES_PER_REQUEST)
        ).await
    }

    /// Fetches Farcaster collectible NFTs, continuing pagination until we get the target number
    /// or exhaust all available NFTs. This helps ensure we don't miss NFTs that have metadata
    /// but appear later in the pagination.
    pub async fn get_farcaster_collectible_casts_with_pagination(&self, owner_address: &str, target_count: u32) -> Result<Vec<ContractTokenPair>> {
        // Rate limiting
        let mut rate_limiter = self.rate_limiter.lock().await;
        if !rate_limiter.can_make_request(owner_address).await {
            return Err(anyhow::anyhow!("Rate limit exceeded for address: {}", owner_address));
        }
        drop(rate_limiter);

        let mut all_nfts = Vec::new();
        let mut page_key: Option<String> = None;
        let mut total_fetched = 0;
        const MAX_TOTAL_FETCH: u32 = 2000; // Safety limit to prevent infinite loops

        loop {
            let mut url = format!(
                "{}/{}/getNFTsForOwner?owner={}&pageSize={}&withMetadata=false",
                self.base_url, self.api_key, owner_address, ALCHEMY_DEFAULT_PAGE_SIZE
            );

            // Add contract address filter for Farcaster collectible casts
            url.push_str(&format!("&contractAddresses%5B%5D={}", FARCASTER_COLLECTIBLE_CASTS_CONTRACT_ADDRESS));

            if let Some(ref key) = page_key {
                url.push_str(&format!("&pageKey={}", key));
            }

            let response = self.client
                .get(&url)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Alchemy API error: {}", error_text));
            }

            let response_data: AlchemyNftsResponse = response.json().await?;
            
            // Convert NFTs to our format
            for nft in response_data.owned_nfts {
                let token_id_hex = uint256_to_bytes20_hex(&nft.token_id)?;
                all_nfts.push(ContractTokenPair {
                    contract_address: nft.contract_address,
                    token_id_decimal: nft.token_id,
                    token_id_hex,
                    balance: nft.balance,
                    image_url: None,
                    metadata_url: None,
                    author: None,
                });

                total_fetched += 1;
                if total_fetched >= MAX_TOTAL_FETCH {
                    tracing::warn!("Hit maximum fetch limit of {} NFTs for address {}", MAX_TOTAL_FETCH, owner_address);
                    return Ok(all_nfts);
                }
            }

            // Check if we have enough or if there's no next page
            page_key = response_data.page_key;
            if page_key.is_none() || all_nfts.len() >= target_count as usize {
                break;
            }
        }

        Ok(all_nfts)
    }

    pub async fn get_all_relevant_nfts(&self, owner_address: &str) -> Result<(Vec<ContractTokenPair>, Vec<ContractTokenPair>)> {
        let dwrcasts = self.get_dwrcasts_nfts(owner_address).await?;
        let farcaster_collectibles = self.get_farcaster_collectible_casts(owner_address).await?;
        
        Ok((dwrcasts, farcaster_collectibles))
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
