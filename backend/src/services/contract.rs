use anyhow::Result;
use alloy::{
    primitives::{Address, U256, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::{TransactionRequest, TransactionInput},
    signers::local::PrivateKeySigner,
    sol_types::SolCall,
};

use alloy::network::TransactionBuilder;
use serde::Deserialize;

use crate::constants::WORLDCHAIN_MAINNET_CHAIN_ID;

#[derive(Debug, Deserialize)]
struct AlchemyGasPriceResponse {
    jsonrpc: String,
    id: u32,
    result: String, // Hex string like "0x10ec7c"
}

// Define the Solidity types for the Together contract
alloy::sol! {
    struct AuthData {
        bytes32 nonce;
        uint256 deadline;
        bytes signature;
    }

    function together(address onBehalfOf, address togetherWith, uint256 timestamp, AuthData authData);
}

#[derive(Debug, Clone)]
pub struct ContractService {
    rpc_url: String,
    together_contract_address: Address,
    alchemy_api_key: String,
}

impl ContractService {
    pub async fn new(rpc_url: String, together_contract_address: String, alchemy_api_key: String) -> Result<Self> {
        let together_contract_address = together_contract_address.parse()?;
        
        Ok(Self {
            rpc_url,
            together_contract_address,
            alchemy_api_key,
        })
    }
    
    /// Fetch gas price from API with fallback to network gas price
    async fn get_optimal_gas_price<P: Provider>(&self, provider: &P) -> Result<u128> {
        // Try to fetch from gas station API first
        match self.fetch_gas_price_from_api().await {
            Ok(gas_price) => {
                tracing::debug!("Using gas price from API: {} gwei", gas_price / 1_000_000_000);
                Ok(gas_price)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch gas price from API: {}, falling back to network price", e);
                // Fallback to network gas price
                let network_gas_price = provider.get_gas_price().await?;
                // Add 10% buffer for faster inclusion
                let buffered_price = network_gas_price * 11 / 10;
                tracing::debug!("Using network gas price with buffer: {} gwei", buffered_price / 1_000_000_000);
                Ok(buffered_price)
            }
        }
    }
    
    /// Fetch gas price from Alchemy API
    async fn fetch_gas_price_from_api(&self) -> Result<u128> {
        let client = reqwest::Client::new();
        
        // Build Alchemy URL with API key
        let alchemy_url = format!("https://worldchain-mainnet.g.alchemy.com/v2/{}", self.alchemy_api_key);
        
        // Create JSON-RPC request payload
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });
        
        let response = client
            .post(&alchemy_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;
        
        let gas_data: AlchemyGasPriceResponse = response.json().await?;
        
        // Parse hex string to u128 (result is in wei)
        let gas_price_hex = gas_data.result.trim_start_matches("0x");
        let gas_price_wei = u128::from_str_radix(gas_price_hex, 16)
            .map_err(|e| anyhow::anyhow!("Failed to parse gas price hex: {}", e))?;
        
        // Apply 1.1x multiplier for faster inclusion
        let fast_price_wei = gas_price_wei * 11 / 10;
        
        tracing::debug!(
            "Alchemy gas price: {} wei ({} gwei), with 1.1x: {} wei ({} gwei)",
            gas_price_wei,
            gas_price_wei / 1_000_000_000,
            fast_price_wei,
            fast_price_wei / 1_000_000_000
        );
        
        Ok(fast_price_wei)
    }
    
    /// Get nonce with retry logic for race conditions
    async fn get_nonce_with_retry<P: Provider>(&self, provider: &P, address: Address, max_retries: u32) -> Result<u64> {
        for attempt in 0..max_retries {
            match provider.get_transaction_count(address).await {
                Ok(nonce) => {
                    if attempt > 0 {
                        tracing::info!("Successfully got nonce {} on attempt {}", nonce, attempt + 1);
                    }
                    return Ok(nonce);
                }
                Err(e) => {
                    if attempt == max_retries - 1 {
                        return Err(e.into());
                    }
                    tracing::warn!("Failed to get nonce on attempt {}: {}, retrying...", attempt + 1, e);
                    tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                }
            }
        }
        unreachable!()
    }
    
    fn create_provider(&self) -> Result<impl Provider> {
        let provider = ProviderBuilder::new().connect_http(self.rpc_url.parse()?);
        Ok(provider)
    }
    
    pub async fn get_latest_block(&self) -> Result<u64> {
        let provider = self.create_provider()?;
        let block = provider.get_block_number().await?;
        Ok(block)
    }
    
    /// Submit a together transaction on behalf of users (server-side signing)
    pub async fn submit_together_transaction_server_signed(
        &self,
        private_key: &str,
        address_1: &str,
        address_2: &str,
        timestamp: u64,
    ) -> Result<String> {
        use crate::utils::eip712::Eip712Signer;
        
        // Parse addresses
        let addr_1: Address = address_1.parse()?;
        let addr_2: Address = address_2.parse()?;
        let contract_address: Address = self.together_contract_address;
        
        // Generate nonce and deadline
        let nonce = Eip712Signer::generate_nonce();
        let deadline = Eip712Signer::generate_deadline_10_minutes();
        
        // Create EIP712 signer
        let signer = Eip712Signer::new(private_key, WORLDCHAIN_MAINNET_CHAIN_ID)?; // Worldchain mainnet chain ID
        
        // Sign the together attestation
        let signature_data = signer.sign_together_attestation(
            contract_address,
            addr_1,
            addr_2,
            timestamp as i64,
            nonce,
            deadline,
        ).await?;
        
        // Submit the transaction
        self.submit_together_transaction(
            private_key,
            addr_1,
            addr_2,
            U256::from(timestamp),
            nonce,
            deadline,
            signature_data.signature,
        ).await
    }
    
    pub async fn submit_together_transaction(
        &self,
        private_key: &str,
        on_behalf_of: Address,
        together_with: Address,
        timestamp: U256,
        nonce: U256,
        deadline: u64,
        signature: String,
    ) -> Result<String> {
        const MAX_RETRY_ATTEMPTS: u32 = 3;
        const NONCE_RETRY_ATTEMPTS: u32 = 3;
        
        let signer: PrivateKeySigner = private_key.parse()?;
        
        // Create provider with wallet for transaction signing
        let provider = ProviderBuilder::new()
            .wallet(signer.clone())
            .connect_http(self.rpc_url.parse()?);
        
        tracing::info!(
            "Starting together transaction submission for {} and {}",
            on_behalf_of,
            together_with
        );
        
        // Parse signature bytes
        let sig_bytes = if signature.starts_with("0x") {
            hex::decode(&signature[2..]).map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?
        } else {
            hex::decode(&signature).map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?
        };
        
        // Create AuthData struct
        let auth_data = AuthData {
            nonce: nonce.into(),
            deadline: U256::from(deadline),
            signature: sig_bytes.into(),
        };
        
        // Create the function call
        let call = togetherCall {
            onBehalfOf: on_behalf_of,
            togetherWith: together_with,
            timestamp,
            authData: auth_data,
        };
        
        // Encode the call data
        let call_data = call.abi_encode();
        
        tracing::debug!(
            "Together call data: 0x{}",
            hex::encode(&call_data)
        );
        
        // Get optimal gas price
        let gas_price = self.get_optimal_gas_price(&provider).await?;
        
        for attempt in 0..MAX_RETRY_ATTEMPTS {
            // Get current nonce with retry logic
            let tx_nonce = self.get_nonce_with_retry(&provider, signer.address(), NONCE_RETRY_ATTEMPTS).await?;
            
            tracing::info!(
                "Attempt {} - Using nonce {} with gas price {} gwei",
                attempt + 1,
                tx_nonce,
                gas_price / 1_000_000_000
            );
            
            // Create base transaction for gas estimation
            let mut tx_base = TransactionRequest::default()
                .to(self.together_contract_address)
                .nonce(tx_nonce)
                .value(U256::ZERO)
                .input(TransactionInput::new(Bytes::from(call_data.clone())))
                .gas_limit(2_000_000u64); // High limit for estimation
            tx_base.set_gas_price(gas_price);
            
            // Estimate gas
            let estimated_gas = match tokio::time::timeout(
                std::time::Duration::from_secs(15),
                provider.estimate_gas(tx_base.clone())
            ).await {
                Ok(Ok(gas)) => gas,
                Ok(Err(e)) => {
                    tracing::error!("Gas estimation failed on attempt {}: {}", attempt + 1, e);
                    if attempt == MAX_RETRY_ATTEMPTS - 1 {
                        return Err(anyhow::anyhow!("Gas estimation failed after {} attempts: {}", MAX_RETRY_ATTEMPTS, e));
                    }
                    continue;
                }
                Err(_) => {
                    tracing::error!("Gas estimation timed out on attempt {}", attempt + 1);
                    if attempt == MAX_RETRY_ATTEMPTS - 1 {
                        return Err(anyhow::anyhow!("Gas estimation timed out after {} attempts", MAX_RETRY_ATTEMPTS));
                    }
                    continue;
                }
            };
            
            tracing::info!("Gas estimated: {}", estimated_gas);
            
            // Add buffer to gas estimate (1.2x)
            let gas_with_buffer = (estimated_gas as f64 * 1.2) as u64;
            let final_tx = tx_base.gas_limit(gas_with_buffer);
            
            tracing::info!(
                "Attempt {} - Sending transaction with gas limit: {}",
                attempt + 1,
                gas_with_buffer
            );
            
            // Send the transaction
            match provider.send_transaction(final_tx).await {
                Ok(pending_tx) => {
                    let tx_hash = *pending_tx.tx_hash();
                    tracing::info!("Together transaction sent with hash: 0x{:x}", tx_hash);
                    
                    // Wait for confirmation with timeout
                    let receipt_future = pending_tx
                        .with_required_confirmations(1)
                        .with_timeout(Some(std::time::Duration::from_secs(30)))
                        .get_receipt();
                    
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        receipt_future
                    ).await {
                        Ok(Ok(receipt)) => {
                            if receipt.status() {
                                tracing::info!(
                                    "Together transaction confirmed successfully: 0x{:x}",
                                    receipt.transaction_hash
                                );
                            } else {
                                tracing::error!(
                                    "Together transaction reverted: 0x{:x}",
                                    receipt.transaction_hash
                                );
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(
                                "Error waiting for together transaction confirmation: {}",
                                e
                            );
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Timeout waiting for together transaction confirmation: 0x{:x}",
                                tx_hash
                            );
                        }
                    }
                    
                    return Ok(format!("0x{:x}", tx_hash));
                }
                Err(e) => {
                    let error_str = e.to_string().to_lowercase();
                    if error_str.contains("replacement transaction underpriced") 
                       || error_str.contains("nonce too low") 
                       || error_str.contains("already known") {
                        tracing::warn!(
                            "Nonce conflict detected on attempt {}: {}. Retrying with fresh nonce...",
                            attempt + 1,
                            e
                        );
                        // Add a small delay before retrying
                        tokio::time::sleep(std::time::Duration::from_millis(200 * (attempt + 1) as u64)).await;
                        continue;
                    } else {
                        tracing::error!("Failed to send transaction on attempt {}: {}", attempt + 1, e);
                        if attempt == MAX_RETRY_ATTEMPTS - 1 {
                            return Err(anyhow::anyhow!("Failed to send transaction after {} attempts: {}", MAX_RETRY_ATTEMPTS, e));
                        }
                        continue;
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Transaction failed after {} retry attempts", MAX_RETRY_ATTEMPTS))
    }
    
    // Add more contract interaction methods as needed
}
