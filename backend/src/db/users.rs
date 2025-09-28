use crate::models::{User, PendingConnection, OptimisticConnection, PendingConnectionMatch};
use anyhow::Result;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

// User operations
pub async fn create_user(pool: &PgPool, wallet_address: &str) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (wallet_address)
        VALUES ($1)
        RETURNING id, wallet_address, created_at, updated_at
        "#,
        wallet_address
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, user_id: i32) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, wallet_address, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_wallet_address(pool: &PgPool, wallet_address: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, wallet_address, created_at, updated_at
        FROM users
        WHERE wallet_address = $1
        "#,
        wallet_address
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_or_create_user(pool: &PgPool, wallet_address: &str) -> Result<User> {
    if let Some(user) = get_user_by_wallet_address(pool, wallet_address).await? {
        Ok(user)
    } else {
        create_user(pool, wallet_address).await
    }
}

// Pending connection operations
pub async fn create_pending_connection(pool: &PgPool, from_user_id: i32, to_user_id: i32) -> Result<PendingConnection> {
    let pending = sqlx::query_as!(
        PendingConnection,
        r#"
        INSERT INTO pending_connections (from_user_id, to_user_id)
        VALUES ($1, $2)
        RETURNING id, from_user_id, to_user_id, created_at, expires_at
        "#,
        from_user_id,
        to_user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(pending)
}

pub async fn get_pending_connection(pool: &PgPool, from_user_id: i32, to_user_id: i32) -> Result<Option<PendingConnection>> {
    let pending = sqlx::query_as!(
        PendingConnection,
        r#"
        SELECT id, from_user_id, to_user_id, created_at, expires_at
        FROM pending_connections
        WHERE from_user_id = $1 AND to_user_id = $2
        AND expires_at > NOW()
        "#,
        from_user_id,
        to_user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(pending)
}

pub async fn find_pending_connection_matches(pool: &PgPool) -> Result<Vec<PendingConnectionMatch>> {
    let matches = sqlx::query!(
        r#"
        SELECT 
            p1.id as p1_id, p1.from_user_id as p1_from, p1.to_user_id as p1_to, 
            p1.created_at as p1_created_at, p1.expires_at as p1_expires_at,
            p2.id as p2_id, p2.from_user_id as p2_from, p2.to_user_id as p2_to,
            p2.created_at as p2_created_at, p2.expires_at as p2_expires_at,
            u1.id as u1_id, u1.wallet_address as u1_address, 
            u1.created_at as u1_created_at, u1.updated_at as u1_updated_at,
            u2.id as u2_id, u2.wallet_address as u2_address,
            u2.created_at as u2_created_at, u2.updated_at as u2_updated_at
        FROM pending_connections p1
        JOIN pending_connections p2 ON p1.from_user_id = p2.to_user_id AND p1.to_user_id = p2.from_user_id
        JOIN users u1 ON p1.from_user_id = u1.id
        JOIN users u2 ON p1.to_user_id = u2.id
        WHERE p1.expires_at > NOW() AND p2.expires_at > NOW()
        AND p1.from_user_id < p1.to_user_id -- Avoid duplicate pairs
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in matches {
        let user_1 = User {
            id: row.u1_id,
            wallet_address: row.u1_address,
            created_at: row.u1_created_at,
            updated_at: row.u1_updated_at,
        };
        
        let user_2 = User {
            id: row.u2_id,
            wallet_address: row.u2_address,
            created_at: row.u2_created_at,
            updated_at: row.u2_updated_at,
        };

        let pending_1 = PendingConnection {
            id: row.p1_id,
            from_user_id: row.p1_from,
            to_user_id: row.p1_to,
            created_at: row.p1_created_at,
            expires_at: row.p1_expires_at,
        };

        let pending_2 = PendingConnection {
            id: row.p2_id,
            from_user_id: row.p2_from,
            to_user_id: row.p2_to,
            created_at: row.p2_created_at,
            expires_at: row.p2_expires_at,
        };

        result.push(PendingConnectionMatch {
            user_1,
            user_2,
            pending_1,
            pending_2,
        });
    }

    Ok(result)
}

pub async fn delete_pending_connection(pool: &PgPool, from_user_id: i32, to_user_id: i32) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM pending_connections
        WHERE from_user_id = $1 AND to_user_id = $2
        "#,
        from_user_id,
        to_user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_expired_pending_connections(pool: &PgPool) -> Result<u64> {
    let result = sqlx::query!(
        r#"
        DELETE FROM pending_connections
        WHERE expires_at <= NOW()
        "#
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// Optimistic connection operations
pub async fn create_optimistic_connection(
    pool: &PgPool, 
    user_id_1: i32, 
    user_id_2: i32
) -> Result<OptimisticConnection> {
    let (smaller_id, larger_id) = if user_id_1 < user_id_2 {
        (user_id_1, user_id_2)
    } else {
        (user_id_2, user_id_1)
    };

    let optimistic = sqlx::query_as!(
        OptimisticConnection,
        r#"
        INSERT INTO optimistic_connections (user_id_1, user_id_2)
        VALUES ($1, $2)
        RETURNING id, user_id_1, user_id_2, processed, created_at
        "#,
        smaller_id,
        larger_id
    )
    .fetch_one(pool)
    .await?;

    Ok(optimistic)
}

pub async fn get_optimistic_connection(pool: &PgPool, user_id_1: i32, user_id_2: i32) -> Result<Option<OptimisticConnection>> {
    let (smaller_id, larger_id) = if user_id_1 < user_id_2 {
        (user_id_1, user_id_2)
    } else {
        (user_id_2, user_id_1)
    };

    let optimistic = sqlx::query_as!(
        OptimisticConnection,
        r#"
        SELECT id, user_id_1, user_id_2, processed, created_at
        FROM optimistic_connections
        WHERE user_id_1 = $1 AND user_id_2 = $2
        "#,
        smaller_id,
        larger_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(optimistic)
}

pub async fn mark_optimistic_connection_processed(pool: &PgPool, user_id_1: i32, user_id_2: i32) -> Result<()> {
    let (smaller_id, larger_id) = if user_id_1 < user_id_2 {
        (user_id_1, user_id_2)
    } else {
        (user_id_2, user_id_1)
    };

    sqlx::query!(
        r#"
        UPDATE optimistic_connections
        SET processed = TRUE
        WHERE user_id_1 = $1 AND user_id_2 = $2
        "#,
        smaller_id,
        larger_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_unprocessed_optimistic_connections(pool: &PgPool) -> Result<Vec<OptimisticConnection>> {
    let connections = sqlx::query_as!(
        OptimisticConnection,
        r#"
        SELECT id, user_id_1, user_id_2, processed, created_at
        FROM optimistic_connections
        WHERE processed = FALSE
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(connections)
}

pub async fn count_unprocessed_optimistic_connections(pool: &PgPool, user_id_1: i32, user_id_2: i32) -> Result<i64> {
    let (smaller_id, larger_id) = if user_id_1 < user_id_2 {
        (user_id_1, user_id_2)
    } else {
        (user_id_2, user_id_1)
    };

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM optimistic_connections
        WHERE user_id_1 = $1 AND user_id_2 = $2 AND processed = FALSE
        "#,
        smaller_id,
        larger_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn delete_pending_connection_by_id(pool: &PgPool, connection_id: uuid::Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM pending_connections
        WHERE id = $1
        "#,
        connection_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_oldest_optimistic_connection_processed(pool: &PgPool, user_id_1: i32, user_id_2: i32) -> Result<()> {
    let (smaller_id, larger_id) = if user_id_1 < user_id_2 {
        (user_id_1, user_id_2)
    } else {
        (user_id_2, user_id_1)
    };

    sqlx::query!(
        r#"
        UPDATE optimistic_connections
        SET processed = TRUE
        WHERE id = (
            SELECT id FROM optimistic_connections
            WHERE user_id_1 = $1 AND user_id_2 = $2 AND processed = FALSE
            ORDER BY created_at ASC
            LIMIT 1
        )
        "#,
        smaller_id,
        larger_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
