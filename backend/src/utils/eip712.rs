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
pub struct TogetherSignatureData {
    pub signature: String,
    pub nonce: U256,
    pub deadline: u64,
    pub on_behalf_of: Address,
    pub together_with: Address,
    pub timestamp: U256,
}

// Define the typed data structures
alloy::sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct TogetherData {
        address onBehalfOf;
        address togetherWith;
        uint256 timestamp;
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
            name: Some(TOGETHER_DOMAIN_NAME.to_string().into()),
            version: Some(TOGETHER_DOMAIN_VERSION.to_string().into()),
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: None, // Will be set when needed
            salt: None,
        };

        Ok(Self { signer, domain })
    }

    pub async fn sign_together_permit(
        &self,
        contract_address: Address,
        on_behalf_of: Address,
        together_with: Address,
        timestamp: U256,
        nonce: U256,
        deadline: u64,
    ) -> Result<TogetherSignatureData> {
        let together_data = TogetherData {
            onBehalfOf: on_behalf_of,
            togetherWith: together_with,
            timestamp: timestamp,
            nonce: nonce.into(),
            deadline: U256::from(deadline),
        };

        // Set the verifying contract for this signature
        let domain = Eip712Domain {
            verifying_contract: Some(contract_address),
            ..self.domain.clone()
        };

        let encoded = together_data.eip712_signing_hash(&domain);
        let signature = self.signer.sign_hash(&encoded).await?;

        Ok(TogetherSignatureData {
            signature: signature.to_string(),
            nonce,
            deadline,
            on_behalf_of: on_behalf_of,
            together_with: together_with,
            timestamp: timestamp,
        })
    }

    pub async fn sign_together_attestation(
        &self,
        contract_address: Address,
        my_address: Address,
        partner_address: Address,
        timestamp: i64,
        nonce: U256,
        deadline: u64,
    ) -> Result<TogetherSignatureData> {
        // Convert i64 timestamp to U256
        let timestamp_u256 = U256::from(timestamp as u64);
        
        // Call the existing permit method
        self.sign_together_permit(
            contract_address,
            my_address,
            partner_address,
            timestamp_u256,
            nonce,
            deadline,
        ).await
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

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
        let signer = Eip712Signer::new(&private_key, WORLDCHAIN_MAINNET_CHAIN_ID)
            .expect("Failed to create EIP712 signer");

        // Real test data for end-to-end test with Plotchy wallet
        let plotchy_wallet: Address = "0xAefC770D8515C552C952a30e597d9fbEa99aA756".parse().unwrap();
        let together_with: Address = "0x59888BE579194C701F16a9425f57ECce3906AF4b".parse().unwrap();
        let timestamp = U256::from_str_radix("1727337600", 10).unwrap();
        let nonce = U256::from_str_radix("2222222222222222222222222222222222222222222222222222222222222222", 16).unwrap();
        let deadline = 1850000000; // Wed Aug 16 2028

        println!("\n=== EIP-712 Signature Test Data ===");
        println!("Signer Address: {}", signer.signer.address());
        println!("Contract Address: {}", contract_address);
        println!("Chain ID: {}", WORLDCHAIN_MAINNET_CHAIN_ID);
        println!("Domain Name: {}", TOGETHER_DOMAIN_NAME);
        println!("Domain Version: {}", TOGETHER_DOMAIN_VERSION);
        println!("Plotchy Wallet: {}", plotchy_wallet);
        println!("Together With: {}", together_with);
        println!("Timestamp: {}", timestamp);
        println!("Nonce: {}", nonce);
        println!("Deadline: {}", deadline);

        // Test Together Signature
        println!("\n=== TOGETHER SIGNATURE ===");
        let together_sig = signer.sign_together_permit(
            contract_address,
            plotchy_wallet,
            together_with,
            timestamp,
            nonce,
            deadline,
        ).await.expect("Failed to create together signature");

        println!("Together Signature: {}", together_sig.signature);
        println!("Together Nonce: {}", together_sig.nonce);
        println!("Together Deadline: {}", together_sig.deadline);

        println!("\n=== SUCCESS ===");
        println!("Together signature generated successfully!");
        println!("Use the data above in your Solidity test to verify signature compatibility.");
    }
}
