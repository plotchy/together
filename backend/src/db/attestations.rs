use anyhow::Result;
use sqlx::PgPool;
use crate::models::attestations::{TogetherAttestation, UserProfile, ConnectionInfo, UsernameCache};

/// Insert a new together attestation
pub async fn insert_attestation(
    pool: &PgPool,
    address_1: &str,
    address_2: &str,
    attestation_timestamp: i64,
    tx_hash: Option<&str>,
    block_number: Option<i64>,
) -> Result<TogetherAttestation> {
    // Ensure consistent ordering (address_1 <= address_2 lexicographically)
    let (addr1, addr2) = if address_1.to_lowercase() <= address_2.to_lowercase() {
        (address_1, address_2)
    } else {
        (address_2, address_1)
    };

    let attestation = sqlx::query_as::<_, TogetherAttestation>(
        r#"
        INSERT INTO together_attestations (address_1, address_2, attestation_timestamp, tx_hash, block_number)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (address_1, address_2, attestation_timestamp) DO NOTHING
        RETURNING *
        "#
    )
    .bind(addr1)
    .bind(addr2)
    .bind(attestation_timestamp)
    .bind(tx_hash)
    .bind(block_number)
    .fetch_one(pool)
    .await?;

    // Update counts for both addresses
    update_address_count(pool, addr1).await?;
    update_address_count(pool, addr2).await?;

    Ok(attestation)
}

/// Update the count for a specific address
async fn update_address_count(pool: &PgPool, address: &str) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO together_counts (address, total_count)
        VALUES ($1, (
            SELECT COUNT(*) FROM together_attestations 
            WHERE LOWER(address_1) = LOWER($1) OR LOWER(address_2) = LOWER($1)
        ))
        ON CONFLICT (address) DO UPDATE SET
            total_count = (
                SELECT COUNT(*) FROM together_attestations 
                WHERE LOWER(address_1) = LOWER($1) OR LOWER(address_2) = LOWER($1)
            ),
            updated_at = NOW()
        "#
    )
    .bind(address)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get total together count for an address
pub async fn get_together_count(pool: &PgPool, address: &str) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT total_count FROM together_counts WHERE LOWER(address) = LOWER($1)"
    )
    .bind(address)
    .fetch_optional(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

/// Get user profile with connections data
pub async fn get_user_profile(pool: &PgPool, address: &str, limit: Option<i64>) -> Result<UserProfile> {
    let total_connections = get_together_count(pool, address).await?;
    
    // Get user's own username
    let user_cache = get_username_cache(pool, address).await?;
    
    let limit = limit.unwrap_or(50); // Default to 50 recent connections
    
    let recent_connections = sqlx::query_as::<_, ConnectionInfo>(
        r#"
        SELECT 
            CASE 
                WHEN LOWER(ta.address_1) = LOWER($1) THEN ta.address_2
                ELSE ta.address_1
            END as partner_address,
            ta.attestation_timestamp,
            ta.tx_hash,
            uc.username as partner_username
        FROM together_attestations ta
        LEFT JOIN username_cache uc ON (
            CASE 
                WHEN LOWER(ta.address_1) = LOWER($1) THEN LOWER(uc.address) = LOWER(ta.address_2)
                ELSE LOWER(uc.address) = LOWER(ta.address_1)
            END
        )
        WHERE LOWER(ta.address_1) = LOWER($1) OR LOWER(ta.address_2) = LOWER($1)
        ORDER BY ta.attestation_timestamp DESC
        LIMIT $2
        "#
    )
    .bind(address)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(UserProfile {
        address: address.to_string(),
        username: user_cache.as_ref().and_then(|cache| cache.username.clone()),
        profile_picture_url: user_cache.as_ref().and_then(|cache| cache.profile_picture_url.clone()),
        total_connections,
        recent_connections,
    })
}

/// Check if two addresses have been together
pub async fn check_together(pool: &PgPool, address_1: &str, address_2: &str) -> Result<Option<TogetherAttestation>> {
    // Ensure consistent ordering
    let (addr1, addr2) = if address_1.to_lowercase() <= address_2.to_lowercase() {
        (address_1, address_2)
    } else {
        (address_2, address_1)
    };

    let attestation = sqlx::query_as::<_, TogetherAttestation>(
        r#"
        SELECT * FROM together_attestations
        WHERE LOWER(address_1) = LOWER($1) AND LOWER(address_2) = LOWER($2)
        ORDER BY attestation_timestamp DESC
        LIMIT 1
        "#
    )
    .bind(addr1)
    .bind(addr2)
    .fetch_optional(pool)
    .await?;

    Ok(attestation)
}

/// Get all attestations for an address with pagination
pub async fn get_attestations_for_address(
    pool: &PgPool, 
    address: &str, 
    offset: Option<i64>, 
    limit: Option<i64>
) -> Result<Vec<TogetherAttestation>> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50);

    let attestations = sqlx::query_as::<_, TogetherAttestation>(
        r#"
        SELECT * FROM together_attestations
        WHERE LOWER(address_1) = LOWER($1) OR LOWER(address_2) = LOWER($1)
        ORDER BY attestation_timestamp DESC
        OFFSET $2 LIMIT $3
        "#
    )
    .bind(address)
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(attestations)
}

/// Get watcher state for resuming blockchain watching
pub async fn get_watcher_state(pool: &PgPool, watcher_id: &str) -> Result<Option<crate::models::attestations::WatcherState>> {
    let state = sqlx::query_as::<_, crate::models::attestations::WatcherState>(
        "SELECT * FROM watcher_state WHERE id = $1"
    )
    .bind(watcher_id)
    .fetch_optional(pool)
    .await?;

    Ok(state)
}

/// Update watcher state
pub async fn update_watcher_state(
    pool: &PgPool,
    watcher_id: &str,
    last_processed_block: i64,
    chunk_size: Option<i64>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO watcher_state (id, last_processed_block, chunk_size)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET
            last_processed_block = $2,
            chunk_size = COALESCE($3, watcher_state.chunk_size),
            updated_at = NOW()
        "#
    )
    .bind(watcher_id)
    .bind(last_processed_block)
    .bind(chunk_size.unwrap_or(500))
    .execute(pool)
    .await?;

    Ok(())
}

/// Get username cache for an address
pub async fn get_username_cache(pool: &PgPool, address: &str) -> Result<Option<UsernameCache>> {
    let cache = sqlx::query_as::<_, UsernameCache>(
        "SELECT * FROM username_cache WHERE LOWER(address) = LOWER($1)"
    )
    .bind(address)
    .fetch_optional(pool)
    .await?;

    Ok(cache)
}

/// Upsert username cache (create or update)
pub async fn upsert_username_cache(
    pool: &PgPool,
    address: &str,
    username: Option<&str>,
    profile_picture_url: Option<&str>,
) -> Result<UsernameCache> {
    let cache = sqlx::query_as::<_, UsernameCache>(
        r#"
        INSERT INTO username_cache (address, username, profile_picture_url)
        VALUES ($1, $2, $3)
        ON CONFLICT (address) DO UPDATE SET
            username = COALESCE($2, username_cache.username),
            profile_picture_url = COALESCE($3, username_cache.profile_picture_url),
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(address)
    .bind(username)
    .bind(profile_picture_url)
    .fetch_one(pool)
    .await?;

    Ok(cache)
}

/// Bulk upsert username cache for multiple addresses
pub async fn bulk_upsert_username_cache(
    pool: &PgPool,
    entries: &[(String, Option<String>, Option<String>)], // (address, username, profile_picture_url)
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO username_cache (address, username, profile_picture_url) "
    );
    
    query_builder.push_values(entries, |mut b, (address, username, profile_picture_url)| {
        b.push_bind(address)
         .push_bind(username)
         .push_bind(profile_picture_url);
    });
    
    query_builder.push(
        " ON CONFLICT (address) DO UPDATE SET 
          username = COALESCE(EXCLUDED.username, username_cache.username),
          profile_picture_url = COALESCE(EXCLUDED.profile_picture_url, username_cache.profile_picture_url),
          updated_at = NOW()"
    );

    query_builder.build().execute(pool).await?;
    
    Ok(())
}
