use together::{
    db::{get_db_pool, DatabaseConfig, users},
    utils::{init_logging, config::Config},
    services::contract::ContractService,
};
use anyhow::Result;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    
    info!("🔗 Starting Together Connection Checker...");
    
    // Load config and connect to database
    let config = Config::from_env()?;
    let db_config = DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    // Setup contract service
    let contract_service = ContractService::new(
        config.rpc_url.clone(),
        config.together_contract_address.clone(),
        config.alchemy_api_key.clone(),
    ).await?;
    
    // Run the connection checker
    run_connection_checker(pool, contract_service, config).await?;
    
    Ok(())
}

async fn run_connection_checker(pool: PgPool, contract_service: ContractService, config: Config) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(5)); // Check every 5 seconds
    let mut iter_count: usize = 0;
    
    loop {
        interval.tick().await;
        iter_count += 1;
        
        info!("🔍 Connection checker iteration {}", iter_count);
        
        // 1. Clean up expired pending connections
        match users::delete_expired_pending_connections(&pool).await {
            Ok(deleted) => {
                if deleted > 0 {
                    info!("🧹 Cleaned up {} expired pending connections", deleted);
                }
            }
            Err(e) => {
                error!("❌ Failed to clean up expired pending connections: {}", e);
            }
        }
        
        // 2. Log unprocessed optimistic connections (for monitoring)
        if iter_count % 60 == 0 { // Log every 5 minutes
            match users::get_unprocessed_optimistic_connections(&pool).await {
                Ok(unprocessed) => {
                    if !unprocessed.is_empty() {
                        info!("📊 {} unprocessed optimistic connections waiting for on-chain verification", unprocessed.len());
                    }
                }
                Err(e) => {
                    error!("❌ Failed to get unprocessed optimistic connections: {}", e);
                }
            }
        }
        
        // 3. Find pending connection matches
        match users::find_pending_connection_matches(&pool).await {
            Ok(matches) => {
                if !matches.is_empty() {
                    info!("🎯 Found {} pending connection matches", matches.len());
                    
                    for connection_match in matches {
                        if let Err(e) = process_connection_match(&pool, &contract_service, &config, connection_match).await {
                            error!("❌ Failed to process connection match: {}", e);
                        }
                    }
                } else if iter_count % 12 == 0 { // Log every minute when no matches
                    info!("📊 No pending connection matches found");
                }
            }
            Err(e) => {
                error!("❌ Failed to find pending connection matches: {}", e);
            }
        }
    }
}

async fn process_connection_match(
    pool: &PgPool,
    contract_service: &ContractService,
    config: &Config,
    connection_match: together::models::PendingConnectionMatch,
) -> Result<()> {
    let user_1 = &connection_match.user_1;
    let user_2 = &connection_match.user_2;
    
    info!(
        "👫 Processing connection match: User {} ({}) <-> User {} ({})",
        user_1.id, user_1.wallet_address,
        user_2.id, user_2.wallet_address
    );
    
    // Check if there are too many unprocessed optimistic connections for these users
    let unprocessed_count = users::count_unprocessed_optimistic_connections(pool, user_1.id, user_2.id).await?;
    if unprocessed_count >= 50 {
        info!("⏳ Too many unprocessed optimistic connections ({}) between users, skipping", unprocessed_count);
        return Ok(());
    }
    
    // Create optimistic connection first (shows users they're connected while tx is pending)
    match users::create_optimistic_connection(pool, user_1.id, user_2.id).await {
        Ok(optimistic) => {
            info!("🎯 Created optimistic connection with ID: {}", optimistic.id);
        }
        Err(e) => {
            error!("❌ Failed to create optimistic connection: {}", e);
            return Err(e);
        }
    }
    
    // Delete the specific pending connections that matched (by ID)
    if let Err(e) = users::delete_pending_connection_by_id(pool, connection_match.pending_1.id).await {
        error!("❌ Failed to delete pending connection 1: {}", e);
    }
    
    if let Err(e) = users::delete_pending_connection_by_id(pool, connection_match.pending_2.id).await {
        error!("❌ Failed to delete pending connection 2: {}", e);
    }
    
    info!("🧹 Cleaned up matched pending connections");
    
    // Now send transaction to contract
    let current_timestamp = Utc::now().timestamp() as u64;
    
    match contract_service.submit_together_transaction_server_signed(
        &config.private_key_signer,
        &user_1.wallet_address,
        &user_2.wallet_address,
        current_timestamp,
    ).await {
        Ok(tx_hash) => {
            info!("✅ Successfully sent attestation transaction: {}", tx_hash);
        }
        Err(e) => {
            error!("❌ Failed to send attestation transaction: {}", e);
            
            // The optimistic connection will expire naturally if the tx never gets sent
            // This way users still see they're "connected" for a bit even if tx fails
        }
    }
    
    Ok(())
}
