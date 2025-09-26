use alloy::{
    primitives::{Address, U256},
    signers::{local::PrivateKeySigner, Signer},
    sol_types::{Eip712Domain, SolStruct},
};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapSignatureData {
    pub signature: String,
    pub nonce: U256,
    pub deadline: u64,
    pub wallet_address: Address,
    pub token_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresaleSignatureData {
    pub signature: String,
    pub nonce: U256,
    pub deadline: u64,
    pub buyer: Address,
    pub token_ids: Vec<String>,
}

// Define the typed data structures
alloy::sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct WrapData {
        address to;
        uint256[] tokenIds;
        bytes32 nonce;
        uint256 deadline;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct PresaleData {
        address buyer;
        uint256[] tokenIds;
        bytes32 nonce;
        uint256 deadline;
    }
}

pub struct Eip712Signer {
    signer: PrivateKeySigner,
    domain: Eip712Domain,
}

impl Eip712Signer {
    pub fn new(private_key: &str, chain_id: u64) -> Result<Self> {
        let signer = private_key.parse::<PrivateKeySigner>()?;
        
        let domain = Eip712Domain {
            name: Some(WRAP_DOMAIN_NAME.to_string().into()),
            version: Some(WRAP_DOMAIN_VERSION.to_string().into()),
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: None, // Will be set when needed
            salt: None,
        };

        Ok(Self { signer, domain })
    }

    pub async fn sign_wrap_permit(
        &self,
        contract_address: Address,
        to_address: Address,
        token_ids: Vec<U256>,
        nonce: U256,
        deadline: u64,
    ) -> Result<WrapSignatureData> {
        let wrap_data = WrapData {
            to: to_address,
            tokenIds: token_ids.clone(),
            nonce: nonce.into(),
            deadline: U256::from(deadline),
        };

        // Set the verifying contract for this signature
        let domain = Eip712Domain {
            verifying_contract: Some(contract_address),
            ..self.domain.clone()
        };

        let encoded = wrap_data.eip712_signing_hash(&domain);
        let signature = self.signer.sign_hash(&encoded).await?;

        Ok(WrapSignatureData {
            signature: signature.to_string(),
            nonce,
            deadline,
            wallet_address: to_address,
            token_ids: token_ids.into_iter().map(|id| id.to_string()).collect(),
        })
    }

    pub async fn sign_presale_permit(
        &self,
        contract_address: Address,
        buyer_address: Address,
        token_ids: Vec<U256>,
        nonce: U256,
        deadline: u64,
    ) -> Result<PresaleSignatureData> {
        let presale_data = PresaleData {
            buyer: buyer_address,
            tokenIds: token_ids.clone(),
            nonce: nonce.into(),
            deadline: U256::from(deadline),
        };

        // Set the verifying contract for this signature
        let domain = Eip712Domain {
            verifying_contract: Some(contract_address),
            ..self.domain.clone()
        };

        let encoded = presale_data.eip712_signing_hash(&domain);
        let signature = self.signer.sign_hash(&encoded).await?;

        Ok(PresaleSignatureData {
            signature: signature.to_string(),
            nonce,
            deadline,
            buyer: buyer_address,
            token_ids: token_ids.into_iter().map(|id| id.to_string()).collect(),
        })
    }

    pub fn generate_nonce() -> U256 {
        let part1 = rand::random::<u64>();
        let part2 = rand::random::<u64>();
        let part3 = rand::random::<u64>();
        let part4 = rand::random::<u64>();
        
        // Combine four u64s to create a 256-bit value
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&part1.to_be_bytes());
        bytes[8..16].copy_from_slice(&part2.to_be_bytes());
        bytes[16..24].copy_from_slice(&part3.to_be_bytes());
        bytes[24..32].copy_from_slice(&part4.to_be_bytes());
        
        U256::from_be_bytes(bytes)
    }

    pub fn generate_deadline_10_minutes() -> u64 {
        let now = Utc::now();
        let deadline = now + chrono::Duration::minutes(SIGNATURE_DEADLINE_MINUTES);
        deadline.timestamp() as u64
    }
}

// Helper function to parse token IDs from strings
pub fn parse_token_ids(token_id_strings: &[String]) -> Result<Vec<U256>> {
    token_id_strings
        .iter()
        .map(|s| {
            s.parse::<U256>()
                .map_err(|e| anyhow::anyhow!("Invalid token ID '{}': {}", s, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_token_ids() {
        let token_ids = vec!["123".to_string(), "456".to_string()];
        let parsed = parse_token_ids(&token_ids).unwrap();
        assert_eq!(parsed, vec![U256::from(123), U256::from(456)]);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = Eip712Signer::generate_nonce();
        let nonce2 = Eip712Signer::generate_nonce();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_generate_deadline() {
        let deadline = Eip712Signer::generate_deadline_10_minutes();
        let now = Utc::now().timestamp() as u64;
        assert!(deadline > now);
        assert!(deadline < now + 700); // Should be less than 11.67 minutes
    }

    #[tokio::test]
    async fn test_eip712_signatures_with_env_signer() {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();
        
        // Get private key from environment
        let private_key = env::var("PRIVATE_KEY_SIGNER")
            .expect("PRIVATE_KEY_SIGNER must be set in .env file");
        
        // Get contract address from environment  
        let contract_address: Address = env::var("DWRCASTS_CONTRACT_ADDRESS")
            .expect("DWRCASTS_CONTRACT_ADDRESS must be set in .env file")
            .parse()
            .expect("Invalid contract address format");

        // Create signer
        let signer = Eip712Signer::new(&private_key, BASE_MAINNET_CHAIN_ID)
            .expect("Failed to create EIP712 signer");

        // Real test data for end-to-end test with Plotchy wallet
        let plotchy_wallet: Address = "0xAefC770D8515C552C952a30e597d9fbEa99aA756".parse().unwrap();
        let token_ids = vec![
            U256::from_str_radix("521714322882942037227809931759338433129904973891", 10).unwrap(),
            U256::from_str_radix("233116540652478911954131331570715987634280337797", 10).unwrap()
        ];
        let nonce = U256::from_str_radix("2222222222222222222222222222222222222222222222222222222222222222", 16).unwrap();
        let deadline = 1850000000; // Wed Aug 16 2028

        println!("\n=== EIP-712 Signature Test Data ===");
        println!("Signer Address: {}", signer.signer.address());
        println!("Contract Address: {}", contract_address);
        println!("Chain ID: {}", BASE_MAINNET_CHAIN_ID);
        println!("Domain Name: {}", WRAP_DOMAIN_NAME);
        println!("Domain Version: {}", WRAP_DOMAIN_VERSION);
        println!("Plotchy Wallet: {}", plotchy_wallet);
        println!("Token IDs: {:?}", token_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>());
        println!("Nonce: {}", nonce);
        println!("Deadline: {}", deadline);

        // Test Wrap Signature
        println!("\n=== WRAP SIGNATURE ===");
        let wrap_sig = signer.sign_wrap_permit(
            contract_address,
            plotchy_wallet,
            token_ids.clone(),
            nonce,
            deadline,
        ).await.expect("Failed to create wrap signature");

        println!("Wrap Signature: {}", wrap_sig.signature);
        println!("Wrap Nonce: {}", wrap_sig.nonce);
        println!("Wrap Deadline: {}", wrap_sig.deadline);

        // Test Presale Signature  
        println!("\n=== PRESALE SIGNATURE ===");
        let presale_sig = signer.sign_presale_permit(
            contract_address,
            plotchy_wallet,
            token_ids.clone(),
            nonce,
            deadline,
        ).await.expect("Failed to create presale signature");

        println!("Presale Signature: {}", presale_sig.signature);
        println!("Presale Nonce: {}", presale_sig.nonce);
        println!("Presale Deadline: {}", presale_sig.deadline);

        println!("\n=== SUCCESS ===");
        println!("Both wrap and presale signatures generated successfully!");
        println!("Use the data above in your Solidity test to verify signature compatibility.");
    }
}
