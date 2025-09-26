use anyhow::Result;
use std::env;
use crate::constants::DEFAULT_SERVER_PORT;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub neynar_api_key: String,
    pub rpc_url: String,
    pub auction_contract_address: String,
    pub dwrcasts_contract_address: String,
    pub opensea_api_key: Option<String>,
    pub r2_bucket_url: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
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
            neynar_api_key: env::var("NEYNAR_API_KEY")
                .map_err(|_| anyhow::anyhow!("NEYNAR_API_KEY must be set"))?,
            rpc_url: env::var("FORK_RPC_URL")
                .map_err(|_| anyhow::anyhow!("FORK_RPC_URL must be set"))?,
            auction_contract_address: env::var("AUCTION_CONTRACT_ADDRESS")
                .map_err(|_| anyhow::anyhow!("AUCTION_CONTRACT_ADDRESS must be set"))?,
            dwrcasts_contract_address: env::var("DWRCASTS_CONTRACT_ADDRESS")
                .map_err(|_| anyhow::anyhow!("DWRCASTS_CONTRACT_ADDRESS must be set"))?,
            opensea_api_key: env::var("OPENSEA_API_KEY").ok(),
            r2_bucket_url: env::var("BUCKET_S3_ENDPOINT")
                .map_err(|_| anyhow::anyhow!("BUCKET_S3_ENDPOINT must be set"))?,
            r2_access_key: env::var("BUCKET_S3_ACCESS_KEY_ID")
                .map_err(|_| anyhow::anyhow!("BUCKET_S3_ACCESS_KEY_ID must be set"))?,
            r2_secret_key: env::var("BUCKET_S3_SECRET_ACCESS_KEY")
                .map_err(|_| anyhow::anyhow!("BUCKET_S3_SECRET_ACCESS_KEY must be set"))?,
            alchemy_api_key: env::var("ALCHEMY_API_KEY")
                .map_err(|_| anyhow::anyhow!("ALCHEMY_API_KEY must be set"))?,
            private_key_signer: env::var("PRIVATE_KEY_SIGNER")
                .map_err(|_| anyhow::anyhow!("PRIVATE_KEY_SIGNER must be set"))?,
        })
    }
}
