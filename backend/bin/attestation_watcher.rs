use dwrcasts::{
    utils, Config, get_db_pool, constants::*,
};
use alloy::{
    primitives::{Address, U256, B256},
    providers::{Provider, ProviderBuilder},
    rpc::types::{Filter, Log},
};
use sqlx::PgPool;
use std::{collections::HashSet, time::Duration, env};
use tokio::time;
use tracing::{info, error, warn, debug};
use uuid::Uuid;


// All constants now imported from constants.rs

#[derive(Debug)]
struct AuctionStartedEvent {
    cast_hash: String,
    creator_fid: i64,
}

#[derive(Debug)]
struct AuctionSettledEvent {
    cast_hash: String,
    winner_address: String,
}

#[derive(Debug)]
struct PresaleClaimedEvent {
    token_id: String,
    buyer: String,
    tx_hash: String,
}

#[derive(Debug)]
struct WatcherState {
    last_processed_block: u64,
    chunk_size: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::init_logging();
    
    // Check for reset flag
    let reset_from_start = env::args().any(|arg| arg == "--reset" || arg == "--from-start");
    
    let config = Config::from_env()?;
    let db_config = dwrcasts::db::DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    // Note: Migrations should be run by the main server, not here
    // This allows the auction watcher to restart frequently without migration overhead
    
    let auction_contract_address: Address = config.auction_contract_address.parse()?;
    
    let dwrcasts_contract_address: Address = config.dwrcasts_contract_address.parse()?;
    
    info!("Starting auction watcher...");
    info!("Auction contract address: {}", auction_contract_address);
    info!("DWRCasts contract address: {}", dwrcasts_contract_address);
    info!("RPC URL: {}", config.rpc_url);
    if reset_from_start {
        info!("🔄 RESET FLAG DETECTED - will start from beginning block");
        reset_watcher_state(&pool).await?;
    }
    
    let mut interval = time::interval(Duration::from_secs(AUCTION_WATCHER_FETCH_INTERVAL_SECS));
    
    loop {
        interval.tick().await;
        
        if let Err(e) = watch_auctions(&pool, &config.rpc_url, auction_contract_address, dwrcasts_contract_address).await {
            error!("Error watching auctions: {}", e);
        }
    }
}

async fn watch_auctions(
    pool: &PgPool,
    rpc_url: &str,
    auction_contract_address: Address,
    dwrcasts_contract_address: Address,
) -> anyhow::Result<()> {
    // Load or initialize watcher state
    let state = load_watcher_state(pool).await?;
    let mut from_block = state.last_processed_block + 1; // Resume from next block
    let mut chunk_size = state.chunk_size;
    let mut iter_count = 0;
    let mut failures_in_row = 0;
    let mut seen_events: HashSet<(B256, u64)> = HashSet::new(); // (tx_hash, log_index)
    
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    let latest_block = provider.get_block_number().await?;
    info!("Starting auction watcher from block {} to {} (resumed from state)", from_block, latest_block);
    info!("Initial chunk size: {}", chunk_size);
    
    while from_block <= latest_block {
        // Refresh latest block periodically
        let current_latest = if iter_count % REFRESH_LATEST_BLOCK_EVERY_N_ITERS == 0 {
            match provider.get_block_number().await {
                Ok(block) => {
                    info!("Found new latest block: {}", block);
                    block
                }
                Err(e) => {
                    warn!("Failed to refresh latest block: {}", e);
                    latest_block
                }
            }
        } else {
            latest_block
        };
        
        if from_block > current_latest {
            break;
        }
        
        let to_block = std::cmp::min(from_block + chunk_size - 1, current_latest);
        info!("Processing blocks {} -> {} (chunk_size={})", from_block, to_block, chunk_size);
        
        match process_block_range(pool, rpc_url, auction_contract_address, dwrcasts_contract_address, from_block, to_block, &mut seen_events).await {
            Ok(events_processed) => {
                if events_processed > 0 {
                    info!("  + {} events processed", events_processed);
                }
                
                // Success: grow chunk size  
                failures_in_row = 0;
                if chunk_size < MAX_CHUNK_SIZE {
                    chunk_size = std::cmp::min(chunk_size * 2, MAX_CHUNK_SIZE);
                }
                
                from_block = to_block + 1;
                
                // Save state after successful processing
                save_watcher_state(pool, from_block - 1, chunk_size).await?;
            }
            Err(e) => {
                failures_in_row += 1;
                error!("  ✗ fetch failed: {}", e);
                
                // Shrink chunk size on failure
                if chunk_size <= MIN_CHUNK_SIZE {
                    tokio::time::sleep(Duration::from_secs(std::cmp::min(2 * failures_in_row, 10))).await;
                } else {
                    chunk_size = std::cmp::max(chunk_size / 2, MIN_CHUNK_SIZE);
                    info!("  ↘ reducing chunk to {}", chunk_size);
                }
            }
        }
        
        iter_count += 1;
    }

    // Clean up expired presale designations
    if let Err(e) = dwrcasts::db::presale_nfts::cleanup_expired_designations(pool).await {
        error!("Failed to cleanup expired presale designations: {}", e);
    }
    
    info!("Auction watcher completed block range processing");
    Ok(())
}

async fn process_block_range(
    pool: &PgPool,
    rpc_url: &str,
    auction_contract_address: Address,
    dwrcasts_contract_address: Address,
    from_block: u64,
    to_block: u64,
    seen_events: &mut HashSet<(B256, u64)>,
) -> anyhow::Result<usize> {
    let mut total_processed = 0;
    
    // Fetch auction events
    let auction_events = fetch_auction_logs(rpc_url, auction_contract_address, from_block, to_block).await?;
    info!("📥 Fetched {} auction events from blocks {}-{}", auction_events.len(), from_block, to_block);
    
    // Fetch presale events
    let presale_events = fetch_presale_logs(rpc_url, dwrcasts_contract_address, from_block, to_block).await?;
    info!("📥 Fetched {} presale events from blocks {}-{}", presale_events.len(), from_block, to_block);
    
    let started_topic: B256 = AUCTION_STARTED_TOPIC.parse()?;
    let settled_topic: B256 = AUCTION_SETTLED_TOPIC.parse()?;
    let presale_claimed_topic: B256 = PRESALE_CLAIMED_TOPIC.parse()?;
    
    let mut started_decoded = 0;
    let mut started_decode_failures = 0;
    let mut settled_decoded = 0;
    let mut settled_decode_failures = 0;
    let mut presale_decoded = 0;
    let mut presale_decode_failures = 0;
    
    // Process auction events
    for (i, event) in auction_events.iter().enumerate() {
        let key = (event.transaction_hash.unwrap_or_default(), event.log_index.unwrap_or_default());
        if seen_events.contains(&key) {
            debug!("  ⏭️  Skipping duplicate event {}/{}", i + 1, auction_events.len());
            continue;
        }
        seen_events.insert(key);
        
        if event.topics().len() == 0 {
            warn!("  ⚠️  Event {}/{} has no topics", i + 1, auction_events.len());
            continue;
        }
        
        let event_sig = event.topics()[0];
        
        if event_sig == started_topic {
            debug!("Processing AuctionStarted event {}/{}: tx={:?}, log_index={:?}", 
                   i + 1, auction_events.len(), 
                   event.transaction_hash, event.log_index);
            
            match decode_auction_started_log(&event) {
                Some(started_event) => {
                    started_decoded += 1;
                    debug!("  ✅ Decoded: cast_hash={}, creator_fid={}", started_event.cast_hash, started_event.creator_fid);
                    
                    match process_auction_started(pool, started_event).await {
                        Ok(_) => {
                            debug!("  💾 Successfully inserted to DB");
                            total_processed += 1;
                        }
                        Err(e) => {
                            error!("  ❌ Failed to insert to DB: {}", e);
                            return Err(e);
                        }
                    }
                }
                None => {
                    started_decode_failures += 1;
                    warn!("  ⚠️  Failed to decode AuctionStarted event: topics={:?}, data={:?}", 
                          event.topics(), event.data());
                }
            }
        } else if event_sig == settled_topic {
            debug!("Processing AuctionSettled event {}/{}: tx={:?}, log_index={:?}", 
                   i + 1, auction_events.len(), 
                   event.transaction_hash, event.log_index);
            
            match decode_auction_settled_log(&event) {
                Some(settled_event) => {
                    settled_decoded += 1;
                    debug!("  ✅ Decoded: cast_hash={}, winner_address={}", settled_event.cast_hash, settled_event.winner_address);
                    
                    match process_auction_settled(pool, settled_event).await {
                        Ok(_) => {
                            debug!("  💾 Successfully processed in DB");
                            total_processed += 1;
                        }
                        Err(e) => {
                            error!("  ❌ Failed to process in DB: {}", e);
                            return Err(e);
                        }
                    }
                }
                None => {
                    settled_decode_failures += 1;
                    warn!("  ⚠️  Failed to decode AuctionSettled event: topics={:?}, data={:?}", 
                          event.topics(), event.data());
                }
            }
        } else {
            debug!("  ⚪ Ignoring unknown auction event signature: {:?}", event_sig);
        }
    }

    // Process presale events
    for (i, event) in presale_events.iter().enumerate() {
        let key = (event.transaction_hash.unwrap_or_default(), event.log_index.unwrap_or_default());
        if seen_events.contains(&key) {
            debug!("  ⏭️  Skipping duplicate presale event {}/{}", i + 1, presale_events.len());
            continue;
        }
        seen_events.insert(key);
        
        if event.topics().len() == 0 {
            warn!("  ⚠️  Presale event {}/{} has no topics", i + 1, presale_events.len());
            continue;
        }
        
        let event_sig = event.topics()[0];
        
        if event_sig == presale_claimed_topic {
            debug!("Processing PresaleClaimed event {}/{}: tx={:?}, log_index={:?}", 
                   i + 1, presale_events.len(), 
                   event.transaction_hash, event.log_index);
            
            match decode_presale_claimed_log(&event) {
                Some(presale_event) => {
                    presale_decoded += 1;
                    debug!("  ✅ Decoded: token_id={}, buyer={}", presale_event.token_id, presale_event.buyer);
                    
                    match process_presale_claimed(pool, presale_event).await {
                        Ok(_) => {
                            debug!("  💾 Successfully processed presale claim in DB");
                            total_processed += 1;
                        }
                        Err(e) => {
                            error!("  ❌ Failed to process presale claim in DB: {}", e);
                            return Err(e);
                        }
                    }
                }
                None => {
                    presale_decode_failures += 1;
                    warn!("  ⚠️  Failed to decode PresaleClaimed event: topics={:?}, data={:?}", 
                          event.topics(), event.data());
                }
            }
        } else {
            debug!("  ⚪ Ignoring unknown presale event signature: {:?}", event_sig);
        }
    }
    
    info!("📊 Event summary:");
    info!("  🏛️  Auction: {} AuctionStarted ({} decoded, {} decode failures), {} AuctionSettled ({} decoded, {} decode failures)", 
          started_decoded + started_decode_failures, started_decoded, started_decode_failures,
          settled_decoded + settled_decode_failures, settled_decoded, settled_decode_failures);
    info!("  🛒 Presale: {} PresaleClaimed ({} decoded, {} decode failures)",
          presale_decoded + presale_decode_failures, presale_decoded, presale_decode_failures);
    info!("🎯 Total events processed for this chunk: {}", total_processed);
    
    Ok(total_processed)
}

async fn fetch_auction_logs(
    rpc_url: &str,
    contract_address: Address,
    from_block: u64,
    to_block: u64,
) -> anyhow::Result<Vec<Log>> {
    let started_topic: B256 = AUCTION_STARTED_TOPIC.parse()?;
    let settled_topic: B256 = AUCTION_SETTLED_TOPIC.parse()?;
    
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    
    let topics = vec![started_topic, settled_topic];
    
    // Create filter for both event types
    let filter = Filter::new()
        .address(contract_address)
        .event_signature(topics)
        .from_block(from_block)
        .to_block(to_block);
    
    let logs = provider.get_logs(&filter).await?;
    Ok(logs)
}

async fn fetch_presale_logs(
    rpc_url: &str,
    contract_address: Address,
    from_block: u64,
    to_block: u64,
) -> anyhow::Result<Vec<Log>> {
    let presale_claimed_topic: B256 = PRESALE_CLAIMED_TOPIC.parse()?;
    
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    
    // Create filter for presale events
    let filter = Filter::new()
        .address(contract_address)
        .event_signature(presale_claimed_topic)
        .from_block(from_block)
        .to_block(to_block);
    
    let logs = provider.get_logs(&filter).await?;
    Ok(logs)
}

fn decode_auction_started_log(log: &Log) -> Option<AuctionStartedEvent> {
    debug!("🔍 Decoding AuctionStarted log: block={:?}, tx={:?}, log_index={:?}", 
           log.block_number, log.transaction_hash, log.log_index);
    
    if log.removed {
        debug!("  ❌ Log marked as removed");
        return None;
    }
    
    let topics = log.topics();
    debug!("  📝 Topics count: {}", topics.len());
    
    if topics.len() < 4 {
        warn!("  ❌ Insufficient topics: expected 4, got {}", topics.len());
        return None;
    }
    
    // topics[0] is event signature
    // topics[1] is cast_hash (bytes32)
    // topics[2] is creator (address) 
    // topics[3] is creator_fid (uint96)
    
    debug!("  📋 Topic[0] (signature): {:?}", topics[0]);
    debug!("  📋 Topic[1] (cast_hash): {:?}", topics[1]);
    debug!("  📋 Topic[2] (creator): {:?}", topics[2]);
    debug!("  📋 Topic[3] (creator_fid): {:?}", topics[3]);
    
    let cast_hash = format!("0x{:x}", topics[1]);
    let creator_fid = U256::from_be_bytes(topics[3].0).to::<u64>() as i64;
    
    debug!("  ✅ Decoded cast_hash: {}", cast_hash);
    debug!("  ✅ Decoded creator_fid: {}", creator_fid);
    
    Some(AuctionStartedEvent {
        cast_hash,
        creator_fid,
    })
}

fn decode_auction_settled_log(log: &Log) -> Option<AuctionSettledEvent> {
    debug!("🔍 Decoding AuctionSettled log: block={:?}, tx={:?}, log_index={:?}", 
           log.block_number, log.transaction_hash, log.log_index);
    
    if log.removed {
        debug!("  ❌ Log marked as removed");
        return None;
    }
    
    let topics = log.topics();
    debug!("  📝 Topics count: {}", topics.len());
    
    if topics.len() < 4 {
        warn!("  ❌ Insufficient topics: expected 4, got {}", topics.len());
        return None;
    }
    
    // topics[0] is event signature
    // topics[1] is cast_hash (bytes32)
    // topics[2] is winner_address (address)
    // topics[3] is winner_fid (uint96)
    
    debug!("  📋 Topic[0] (signature): {:?}", topics[0]);
    debug!("  📋 Topic[1] (cast_hash): {:?}", topics[1]);
    debug!("  📋 Topic[2] (winner_address): {:?}", topics[2]);
    debug!("  📋 Topic[3] (winner_fid): {:?}", topics[3]);
    
    let cast_hash = format!("0x{:x}", topics[1]);
    let winner_address = format!("0x{}", hex::encode(topics[2].get(12..).unwrap_or(&[0u8; 20]))); // Extract address from bytes32
    
    debug!("  ✅ Decoded cast_hash: {}", cast_hash);
    debug!("  ✅ Decoded winner_address: {}", winner_address);
    
    Some(AuctionSettledEvent {
        cast_hash,
        winner_address,
    })
}

fn decode_presale_claimed_log(log: &Log) -> Option<PresaleClaimedEvent> {
    debug!("🔍 Decoding PresaleClaimed log: block={:?}, tx={:?}, log_index={:?}", 
           log.block_number, log.transaction_hash, log.log_index);
    
    if log.removed {
        debug!("  ❌ Log marked as removed");
        return None;
    }
    
    let topics = log.topics();
    debug!("  📝 Topics count: {}", topics.len());
    
    if topics.len() < 3 {
        warn!("  ❌ Insufficient topics: expected 3, got {}", topics.len());
        return None;
    }
    
    // topics[0] is event signature
    // topics[1] is buyer (address)
    // topics[2] is token_id (uint256)
    // NOTE: The actual event structure will depend on your smart contract
    // This is a placeholder implementation - update based on your actual event
    
    debug!("  📋 Topic[0] (signature): {:?}", topics[0]);
    debug!("  📋 Topic[1] (buyer): {:?}", topics[1]);
    debug!("  📋 Topic[2] (token_id): {:?}", topics[2]);
    
    let buyer = format!("0x{}", hex::encode(topics[1].get(12..).unwrap_or(&[0u8; 20]))); // Extract address from bytes32
    let token_id = U256::from_be_bytes(topics[2].0).to_string();
    let tx_hash = log.transaction_hash.map(|h| format!("0x{:x}", h)).unwrap_or_default();
    
    debug!("  ✅ Decoded buyer: {}", buyer);
    debug!("  ✅ Decoded token_id: {}", token_id);
    debug!("  ✅ Decoded tx_hash: {}", tx_hash);
    
    Some(PresaleClaimedEvent {
        token_id,
        buyer,
        tx_hash,
    })
}

async fn process_auction_started(pool: &PgPool, event: AuctionStartedEvent) -> anyhow::Result<()> {
    info!("💿 Processing AuctionStarted: cast_hash={}, creator_fid={}", event.cast_hash, event.creator_fid);
    
    let auction_id = Uuid::new_v4();
    debug!("Generated auction UUID: {}", auction_id);
    
    // Insert or update auction record
    let result = sqlx::query!(
        r#"
        INSERT INTO auctions (id, cast_hash, creator_fid, settled, winner_address, updated_at)
        VALUES ($1, $2, $3, false, NULL, NOW())
        ON CONFLICT (cast_hash) 
        DO UPDATE SET 
            creator_fid = EXCLUDED.creator_fid,
            updated_at = NOW()
        "#,
        auction_id,
        event.cast_hash,
        event.creator_fid
    )
    .execute(pool)
    .await?;
    
    info!("✅ AuctionStarted SQL result: {} rows affected", result.rows_affected());
    
    // Verify the insert worked by querying back
    let verification = sqlx::query!(
        "SELECT id, cast_hash, creator_fid, settled, winner_address FROM auctions WHERE cast_hash = $1",
        event.cast_hash
    )
    .fetch_optional(pool)
    .await?;
    
    match verification {
        Some(row) => {
            info!("🔍 Verification successful: id={}, cast_hash={}, creator_fid={:?}, settled={}, winner_address={:?}", 
                  row.id, row.cast_hash, row.creator_fid, row.settled, row.winner_address);
        }
        None => {
            error!("🚨 VERIFICATION FAILED: No auction found with cast_hash={}", event.cast_hash);
        }
    }
    
    Ok(())
}

async fn process_auction_settled(pool: &PgPool, event: AuctionSettledEvent) -> anyhow::Result<()> {
    info!("💿 Processing AuctionSettled: cast_hash={}, winner_address={}", event.cast_hash, event.winner_address);
    
    // Update auction to mark as settled and set winner address
    let result = sqlx::query!(
        r#"
        UPDATE auctions 
        SET settled = true, winner_address = $2, updated_at = NOW()
        WHERE cast_hash = $1
        "#,
        event.cast_hash,
        event.winner_address
    )
    .execute(pool)
    .await?;
    
    info!("✅ AuctionSettled UPDATE result: {} rows affected", result.rows_affected());
    
    if result.rows_affected() == 0 {
        warn!("⚠️  AuctionSettled event for unknown cast_hash: {}", event.cast_hash);
        info!("🔧 Inserting orphaned settled auction...");
        
        let fallback_id = Uuid::new_v4();
        debug!("Generated fallback auction UUID: {}", fallback_id);
        
        // Insert the settled auction even if we missed the started event
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO auctions (id, cast_hash, creator_fid, settled, winner_address, updated_at)
            VALUES ($1, $2, NULL, true, $3, NOW())
            ON CONFLICT (cast_hash) DO NOTHING
            "#,
            fallback_id,
            event.cast_hash,
            event.winner_address
        )
        .execute(pool)
        .await?;
        
        info!("✅ Fallback INSERT result: {} rows affected", insert_result.rows_affected());
    }
    
    // Verify the final state
    let verification = sqlx::query!(
        "SELECT id, cast_hash, creator_fid, settled, winner_address FROM auctions WHERE cast_hash = $1",
        event.cast_hash
    )
    .fetch_optional(pool)
    .await?;
    
    match verification {
        Some(row) => {
            info!("🔍 Verification successful: id={}, cast_hash={}, creator_fid={:?}, settled={}, winner_address={:?}", 
                  row.id, row.cast_hash, row.creator_fid, row.settled, row.winner_address);
        }
        None => {
            error!("🚨 VERIFICATION FAILED: No auction found with cast_hash={}", event.cast_hash);
        }
    }
    
    Ok(())
}

async fn process_presale_claimed(pool: &PgPool, event: PresaleClaimedEvent) -> anyhow::Result<()> {
    info!("💿 Processing PresaleClaimed: token_id={}, buyer={}, tx_hash={}", 
          event.token_id, event.buyer, event.tx_hash);
    
    // Mark the NFT as sold in the database
    let success = dwrcasts::db::presale_nfts::mark_nft_sold(pool, &event.token_id, &event.tx_hash).await?;
    
    if success {
        info!("✅ Successfully marked presale NFT {} as sold to {}", event.token_id, event.buyer);
    } else {
        warn!("⚠️  No presale NFT found with token_id {} to mark as sold", event.token_id);
        // This could happen if:
        // 1. The NFT wasn't part of the presale pool
        // 2. It was already marked as sold
        // 3. There's a timing issue with database updates
    }
    
    Ok(())
}

async fn load_watcher_state(pool: &PgPool) -> anyhow::Result<WatcherState> {
    let row = sqlx::query!(
        "SELECT last_processed_block, chunk_size FROM watcher_state WHERE id = $1",
        AUCTION_WATCHER_ID
    )
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(row) => {
            info!("Loaded watcher state: block={}, chunk_size={}", row.last_processed_block, row.chunk_size);
            Ok(WatcherState {
                last_processed_block: row.last_processed_block as u64,
                chunk_size: row.chunk_size as u64,
            })
        }
        None => {
            info!("No existing watcher state found, starting from block {}", AUCTION_WATCHER_START_BLOCK);
            // Initialize with default state
            let state = WatcherState {
                last_processed_block: AUCTION_WATCHER_START_BLOCK - 1, // Will start from AUCTION_WATCHER_START_BLOCK
                chunk_size: INITIAL_CHUNK_SIZE,
            };
            save_watcher_state(pool, state.last_processed_block, state.chunk_size).await?;
            Ok(state)
        }
    }
}

async fn save_watcher_state(pool: &PgPool, last_processed_block: u64, chunk_size: u64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO watcher_state (id, last_processed_block, chunk_size, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (id) 
        DO UPDATE SET 
            last_processed_block = EXCLUDED.last_processed_block,
            chunk_size = EXCLUDED.chunk_size,
            updated_at = NOW()
        "#,
        AUCTION_WATCHER_ID,
        last_processed_block as i64,
        chunk_size as i64
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn reset_watcher_state(pool: &PgPool) -> anyhow::Result<()> {
    info!("🔄 Resetting watcher state to start from block {}", AUCTION_WATCHER_START_BLOCK);
    
    // Delete existing state to force restart from beginning
    let delete_result = sqlx::query!(
        "DELETE FROM watcher_state WHERE id = $1",
        AUCTION_WATCHER_ID
    )
    .execute(pool)
    .await?;
    
    info!("🗑️  Deleted {} existing watcher state records", delete_result.rows_affected());
    
    // Optionally clear existing auction data (uncomment if you want a clean slate)
    // WARNING: This will delete all auction data!
    // let clear_result = sqlx::query!("DELETE FROM auctions").execute(pool).await?;
    // info!("🗑️  Cleared {} existing auction records", clear_result.rows_affected());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::rpc::types::Log;

    #[test]
    fn test_address_decoding() {
        // Test case from the user's example:
        // topic2: 0x00000000000000000000000063396c40c4d51921f983294df2c92ccca3584f30
        // Expected winner address: 0x63396c40c4d51921f983294df2c92ccca3584f30
        
        let topic2_bytes = hex::decode("00000000000000000000000063396c40c4d51921f983294df2c92ccca3584f30").unwrap();
        let mut topic2_array = [0u8; 32];
        topic2_array.copy_from_slice(&topic2_bytes);
        let topic2 = B256::from(topic2_array);
        
        // Test our address extraction logic
        let extracted_address = format!("0x{}", hex::encode(topic2.get(12..).unwrap_or(&[0u8; 20])));
        let expected_address = "0x63396c40c4d51921f983294df2c92ccca3584f30";
        
        println!("🧪 Testing address extraction:");
        println!("  Input topic2: 0x{}", hex::encode(topic2_bytes));
        println!("  Extracted address: {}", extracted_address);
        println!("  Expected address: {}", expected_address);
        
        assert_eq!(extracted_address.to_lowercase(), expected_address.to_lowercase());
        
        // Test another address to make sure it's working generally
        let topic1_bytes = hex::decode("000000000000000000000000c19c9c976f0557bd3919d92b758a6e414e2ba464").unwrap();
        let mut topic1_array = [0u8; 32];
        topic1_array.copy_from_slice(&topic1_bytes);
        let topic1 = B256::from(topic1_array);
        
        let extracted_address2 = format!("0x{}", hex::encode(topic1.get(12..).unwrap_or(&[0u8; 20])));
        let expected_address2 = "0xc19c9c976f0557bd3919d92b758a6e414e2ba464";
        
        println!("🧪 Testing second address extraction:");
        println!("  Input topic1: 0x{}", hex::encode(topic1_bytes));
        println!("  Extracted address: {}", extracted_address2);
        println!("  Expected address: {}", expected_address2);
        
        assert_eq!(extracted_address2.to_lowercase(), expected_address2.to_lowercase());
        
        println!("✅ All address extraction tests passed!");
    }
    
}