use anyhow::Result;
use alloy::{
    primitives::{Address, U256, Bytes},
    providers::{Provider, ProviderBuilder},
    rpc::types::{TransactionRequest, TransactionInput},
};

use alloy::primitives::keccak256;

#[derive(Debug, Clone)]
pub struct ContractService {
    rpc_url: String,
    auction_contract_address: Address,
    dwrcasts_contract_address: Address,
}

impl ContractService {
    pub async fn new(rpc_url: String, auction_contract_address: String, dwrcasts_contract_address: String) -> Result<Self> {
        let auction_contract_address = auction_contract_address.parse()?;
        let dwrcasts_contract_address = dwrcasts_contract_address.parse()?;
        
        Ok(Self {
            rpc_url,
            auction_contract_address,
            dwrcasts_contract_address,
        })
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
    
    pub fn auction_contract_address(&self) -> Address {
        self.auction_contract_address
    }
    
    pub async fn get_presale_claimed_count(&self, buyer_address: Address) -> Result<U256> {
        let provider = self.create_provider()?;
        
        // Create the function selector for presaleClaimedCount(address)
        let function_signature = "presaleClaimedCount(address)";
        let selector = &keccak256(function_signature.as_bytes())[..4];
        
        // Encode the function call data
        let mut call_data = Vec::new();
        call_data.extend_from_slice(selector);
        call_data.extend_from_slice(&[0u8; 12]); // Pad address to 32 bytes
        call_data.extend_from_slice(buyer_address.as_slice());
        
        // Make the call
        let tx = TransactionRequest::default()
            .to(self.dwrcasts_contract_address)
            .input(TransactionInput::new(Bytes::from(call_data)));
            
        let result = provider.call(tx).await?;
        
        // Parse the result as U256
        if result.len() >= 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&result[..32]);
            Ok(U256::from_be_bytes(bytes))
        } else {
            Err(anyhow::anyhow!("Invalid response length from contract call"))
        }
    }
    
    // Add more contract interaction methods as needed
}
