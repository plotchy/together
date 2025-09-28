use together::{
    constants::*,
    db::{get_db_pool, DatabaseConfig},
    utils::{init_logging, config::Config},
    db::{attestations, users},
};
use alloy::{
    primitives::{B256, U256, Address},
    providers::{Provider, ProviderBuilder},
    rpc::types::{Filter, Log},
};
use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

#[derive(Debug)]
struct TogetherEvent {
    address_1: String,
    address_2: String,
    timestamp: u64,
    tx_hash: String,
    block_number: u64,
}

#[derive(Debug)]
struct WatcherState {
    last_processed_block: u64,
    chunk_size: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    
    info!("🎯 Starting Together Attestation Watcher...");
    
    // Load config and connect to database
    let config = Config::from_env()?;
    let db_config = DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    // Setup provider
    let provider = ProviderBuilder::new()
        .connect(&config.rpc_url)
        .await?;
    let provider = Arc::new(provider);
    
    // Get contract address
    let contract_address: Address = config.together_contract_address.parse()?;
    
    // Run the watcher
    run_attestation_watcher(provider, pool, contract_address).await?;
    
    Ok(())
}

async fn run_attestation_watcher(
    provider: Arc<impl Provider + 'static>,
    pool: PgPool,
    contract_address: Address,
) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(ATTESTATION_WATCHER_FETCH_INTERVAL_SECS));
    let mut iter_count: usize = 0;
    let mut latest_known_block = get_latest_block(&provider).await?;
    
    loop {
        interval.tick().await;
        iter_count += 1;
        
        // Get watcher state from DB
        let mut watcher_state = get_or_create_watcher_state(&pool).await?;
        
        info!(
            "📊 Watcher iteration {} | Last processed: {} | Latest: {} | Chunk size: {}",
            iter_count, 
            watcher_state.last_processed_block, 
            latest_known_block, 
            watcher_state.chunk_size
        );
        
        // Fetch latest block when caught up, or periodically when behind
        let blocks_behind = latest_known_block.saturating_sub(watcher_state.last_processed_block);
        let should_refresh = blocks_behind <= watcher_state.chunk_size || iter_count % REFRESH_LATEST_BLOCK_EVERY_N_ITERS == 0;
        
        let current_latest = if should_refresh {
            match get_latest_block(&provider).await {
                Ok(latest) => {
                    latest_known_block = latest;
                    latest
                }
                Err(e) => {
                    warn!("Failed to get latest block: {}, using cached value", e);
                    latest_known_block
                }
            }
        } else {
            latest_known_block
        };
        
        // Skip if we're caught up
        if watcher_state.last_processed_block >= current_latest {
            continue;
        }
        
        // Calculate range to process
        let from_block = watcher_state.last_processed_block + 1;
        let to_block = std::cmp::min(
            from_block + watcher_state.chunk_size - 1, 
            current_latest
        );
        
        info!("🔍 Processing blocks {} to {} (chunk size: {})", from_block, to_block, watcher_state.chunk_size);
        
        // Process the block range
        match process_block_range(&provider, &pool, contract_address, from_block, to_block).await {
            Ok(events_processed) => {
                info!("✅ Successfully processed {} Together events in range {} to {}", events_processed, from_block, to_block);
                
                // Update watcher state
                attestations::update_watcher_state(&pool, ATTESTATION_WATCHER_ID, to_block as i64, None).await?;
                watcher_state.last_processed_block = to_block;
                
                // Increase chunk size on success
                if watcher_state.chunk_size < MAX_CHUNK_SIZE {
                    watcher_state.chunk_size = std::cmp::min(watcher_state.chunk_size * 2, MAX_CHUNK_SIZE);
                    attestations::update_watcher_state(&pool, ATTESTATION_WATCHER_ID, to_block as i64, Some(watcher_state.chunk_size as i64)).await?;
                    info!("📈 Increased chunk size to {}", watcher_state.chunk_size);
                }
            }
            Err(e) => {
                error!("❌ Error processing blocks {} to {}: {}", from_block, to_block, e);
                
                // Decrease chunk size on error
                if watcher_state.chunk_size <= MIN_CHUNK_SIZE {
                    error!("⚠️ Chunk size already at minimum ({}), will retry", MIN_CHUNK_SIZE);
                } else {
                    watcher_state.chunk_size = std::cmp::max(watcher_state.chunk_size / 2, MIN_CHUNK_SIZE);
                    attestations::update_watcher_state(&pool, ATTESTATION_WATCHER_ID, watcher_state.last_processed_block as i64, Some(watcher_state.chunk_size as i64)).await?;
                    info!("📉 Decreased chunk size to {}", watcher_state.chunk_size);
                }
                
                // Wait before retrying
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_block_range(
    provider: &impl Provider,
    pool: &PgPool,
    contract_address: Address,
    from_block: u64,
    to_block: u64,
) -> Result<usize> {
    let together_topic: B256 = TOGETHER_EVENT_TOPIC.parse()?;
    
    let filter = Filter::new()
        .address(contract_address)
        .event_signature(together_topic)
        .from_block(from_block)
        .to_block(to_block);
    
    let logs = provider.get_logs(&filter).await?;
    info!("🔍 Found {} logs in blocks {} to {}", logs.len(), from_block, to_block);
    
    let mut events_processed = 0;
    
    for log in logs {
        if let Err(e) = process_together_log(pool, &log).await {
            error!("Failed to process log in tx {}: {}", log.transaction_hash.unwrap_or_default(), e);
        } else {
            events_processed += 1;
        }
    }
    
    Ok(events_processed)
}

async fn process_together_log(pool: &PgPool, log: &Log) -> Result<()> {
    let event = parse_together_event(log)?;
    
    info!(
        "👫 Together event: {} & {} at timestamp {} (tx: {}, block: {})",
        event.address_1,
        event.address_2,
        event.timestamp,
        event.tx_hash,
        event.block_number
    );
    
    // Insert attestation into database
    match attestations::insert_attestation(
        pool,
        &event.address_1,
        &event.address_2,
        event.timestamp as i64,
        Some(&event.tx_hash),
        Some(event.block_number as i64),
    ).await {
        Ok(attestation) => {
            info!("✅ Successfully inserted attestation with ID: {}", attestation.id);
            
            // Try to mark the oldest unprocessed optimistic connection as processed
            // First get users by wallet addresses
            if let (Ok(Some(user1)), Ok(Some(user2))) = (
                users::get_user_by_wallet_address(pool, &event.address_1).await,
                users::get_user_by_wallet_address(pool, &event.address_2).await
            ) {
                if let Err(e) = users::mark_oldest_optimistic_connection_processed(pool, user1.id, user2.id).await {
                    // This is not critical - there might not be any unprocessed optimistic connections
                    // (e.g., if this attestation was created outside our pending connection system)
                    info!("ℹ️ Could not mark optimistic connection as processed: {}", e);
                } else {
                    info!("🔗 Marked oldest optimistic connection as processed for users {} & {}", user1.id, user2.id);
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to insert attestation: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

fn parse_together_event(log: &Log) -> Result<TogetherEvent> {
    // TogetherEvent(address indexed onBehalfOf, address indexed togetherWith, uint256 indexed timestamp)
    if log.topics().len() != 4 {
        return Err(anyhow::anyhow!("Invalid Together event: expected 4 topics, got {}", log.topics().len()));
    }
    
    // Extract addresses and timestamp from topics (32 bytes, last 20 are the address for address topics)
    let topics = log.topics();
    let address_1_bytes = &topics[1].as_slice()[12..32];
    let address_2_bytes = &topics[2].as_slice()[12..32];
    let timestamp_bytes = &topics[3];
    let timestamp = U256::from_be_slice(timestamp_bytes.as_slice()).to::<u64>();
    
    let address_1 = format!("0x{}", hex::encode(address_1_bytes));
    let address_2 = format!("0x{}", hex::encode(address_2_bytes));
    
    let tx_hash = log.transaction_hash
        .ok_or_else(|| anyhow::anyhow!("Missing transaction hash"))?
        .to_string();
    
    let block_number = log.block_number
        .ok_or_else(|| anyhow::anyhow!("Missing block number"))?;
    
    Ok(TogetherEvent {
        address_1,
        address_2,
        timestamp,
        tx_hash,
        block_number,
    })
}

async fn get_latest_block(provider: &impl Provider) -> Result<u64> {
    let block_number = provider.get_block_number().await?;
    Ok(block_number)
}

async fn get_or_create_watcher_state(pool: &PgPool) -> Result<WatcherState> {
    match attestations::get_watcher_state(pool, ATTESTATION_WATCHER_ID).await? {
        Some(state) => {
            info!("📋 Found existing watcher state: block {}, chunk size {}", state.last_processed_block, state.chunk_size);
            Ok(WatcherState {
                last_processed_block: state.last_processed_block as u64,
                chunk_size: state.chunk_size as u64,
            })
        }
        None => {
            info!("📋 No existing watcher state found, starting from block {}", ATTESTATION_WATCHER_START_BLOCK);
            
            // Create initial state
            attestations::update_watcher_state(
                pool,
                ATTESTATION_WATCHER_ID,
                (ATTESTATION_WATCHER_START_BLOCK - 1) as i64, // Will start from ATTESTATION_WATCHER_START_BLOCK
                Some(INITIAL_CHUNK_SIZE as i64),
            ).await?;
            
            Ok(WatcherState {
                last_processed_block: ATTESTATION_WATCHER_START_BLOCK - 1,
                chunk_size: INITIAL_CHUNK_SIZE,
            })
        }
    }
}

// Reset watcher state function (useful for debugging)
#[allow(dead_code)]
async fn reset_watcher_state(pool: &PgPool) -> Result<()> {
    info!("🔄 Resetting watcher state to start from block {}", ATTESTATION_WATCHER_START_BLOCK);
    
    sqlx::query("DELETE FROM watcher_state WHERE id = $1")
        .bind(ATTESTATION_WATCHER_ID)
        .execute(pool)
        .await?;
    
    info!("✅ Watcher state reset successfully");
    Ok(())
}