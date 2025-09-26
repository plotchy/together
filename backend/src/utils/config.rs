use anyhow::Result;
use std::env;
use crate::constants::DEFAULT_SERVER_PORT;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub rpc_url: String,
    pub together_contract_address: String,
    pub alchemy_api_key: String,
    pub private_key_signer: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present
        
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            port: env::var("PORT")
                .unwrap_or_else(|_| DEFAULT_SERVER_PORT.to_string())
                .parse()
                .unwrap_or(DEFAULT_SERVER_PORT),
            rpc_url: env::var("FORK_RPC_URL")
                .map_err(|_| anyhow::anyhow!("FORK_RPC_URL must be set"))?,
            together_contract_address: env::var("TOGETHER_CONTRACT_ADDRESS")
                .map_err(|_| anyhow::anyhow!("TOGETHER_CONTRACT_ADDRESS must be set"))?,
            alchemy_api_key: env::var("ALCHEMY_API_KEY")
                .map_err(|_| anyhow::anyhow!("ALCHEMY_API_KEY must be set"))?,
            private_key_signer: env::var("PRIVATE_KEY_SIGNER")
                .map_err(|_| anyhow::anyhow!("PRIVATE_KEY_SIGNER must be set"))?,
        })
    }
}
