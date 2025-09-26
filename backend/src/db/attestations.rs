use anyhow::Result;
use sqlx::PgPool;

/// Check if a wallet address has ever won an auction
pub async fn check_auction_winner(pool: &PgPool, address: &str) -> Result<bool> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM auctions WHERE LOWER(winner_address) = LOWER($1) AND settled = true)"
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

/// Get count of auctions won by a wallet address
pub async fn get_auctions_won_count(pool: &PgPool, address: &str) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM auctions WHERE LOWER(winner_address) = LOWER($1) AND settled = true"
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    Ok(count)
}
