use anyhow::Result;
use chrono::Utc;
use clap::{Arg, Command};
use dwrcasts::db::{get_db_pool, DatabaseConfig};
use dwrcasts::utils::config::Config;
use serde_json;
use sqlx::{PgPool, Row};
use std::env;
use std::fs;
use std::io::Write;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("backup-and-wipe-metadata")
        .about("Backup and wipe the metadata table")
        .arg(
            Arg::new("backup-only")
                .long("backup-only")
                .help("Only create backup, don't wipe the table")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("confirm-wipe")
                .long("confirm-wipe")
                .help("Confirm that you want to wipe the metadata table (required for wipe)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let backup_only = matches.get_flag("backup-only");
    let confirm_wipe = matches.get_flag("confirm-wipe");

    // Load configuration
    let config = Config::from_env()?;
    
    // Use DATABASE_PUBLIC_URL if available (for prod access), otherwise DATABASE_URL
    let database_url = match env::var("DATABASE_PUBLIC_URL") {
        Ok(public_url) => {
            info!("Using DATABASE_PUBLIC_URL for production database access");
            public_url
        }
        Err(_) => {
            info!("Using DATABASE_URL (DATABASE_PUBLIC_URL not set)");
            config.database_url.clone()
        }
    };

    // Setup database connection
    let db_config = DatabaseConfig {
        database_url,
        max_connections: 5,
    };
    let pool = get_db_pool(&db_config).await?;
    info!("Connected to database");

    // Step 1: Create backup
    info!("🔄 Creating backup of metadata table...");
    let backup_file = create_backup(&pool).await?;
    info!("✅ Backup created successfully: {}", backup_file);

    if backup_only {
        info!("Backup-only mode. Metadata table was not modified.");
        return Ok(());
    }

    // Step 2: Wipe table (only if confirmed)
    if !confirm_wipe {
        warn!("⚠️  Wipe not confirmed. Use --confirm-wipe to proceed with wiping the table.");
        info!("Backup created: {}", backup_file);
        return Ok(());
    }

    info!("🔄 Wiping metadata table...");
    let deleted_count = wipe_metadata_table(&pool).await?;
    info!("✅ Wiped {} records from metadata table", deleted_count);

    info!("🎉 Operation complete!");
    info!("📁 Backup: {}", backup_file);
    info!("🗑️  Deleted: {} records", deleted_count);

    Ok(())
}

async fn create_backup(pool: &PgPool) -> Result<String> {
    // Create backup directory
    fs::create_dir_all("./db_backups")?;
    
    // Generate timestamp for backup filename
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_file = format!("./db_backups/metadata_backup_{}.json", timestamp);
    
    // Query all metadata records
    let rows = sqlx::query("SELECT * FROM metadata ORDER BY created_at")
        .fetch_all(pool)
        .await?;
    
    info!("Found {} metadata records to backup", rows.len());
    
    // Convert rows to JSON
    let mut records = Vec::new();
    for row in rows {
        let record = serde_json::json!({
            "id": row.get::<uuid::Uuid, _>("id").to_string(),
            "cast_hash": row.get::<String, _>("cast_hash"),
            "metadata_url": row.get::<String, _>("metadata_url"),
            "traits": row.get::<serde_json::Value, _>("traits"),
            "image_url": row.get::<String, _>("image_url"),
            "processed": row.get::<bool, _>("processed"),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        });
        records.push(record);
    }
    
    // Write to file
    let json_data = serde_json::to_string_pretty(&records)?;
    let mut file = fs::File::create(&backup_file)?;
    file.write_all(json_data.as_bytes())?;
    
    // Also create a SQL restore script
    let sql_backup_file = format!("./db_backups/metadata_backup_{}.sql", timestamp);
    let mut sql_file = fs::File::create(&sql_backup_file)?;
    
    writeln!(sql_file, "-- Metadata table backup created at {}", Utc::now().to_rfc3339())?;
    writeln!(sql_file, "-- Records: {}", records.len())?;
    writeln!(sql_file, "-- To restore: psql $DATABASE_PUBLIC_URL -f {}", sql_backup_file)?;
    writeln!(sql_file)?;
    
    for record in &records {
        let insert_sql = format!(
            "INSERT INTO metadata (id, cast_hash, metadata_url, traits, image_url, processed, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}', {}, '{}', '{}');",
            record["id"].as_str().unwrap(),
            record["cast_hash"].as_str().unwrap(),
            record["metadata_url"].as_str().unwrap(),
            record["traits"].to_string().replace("'", "''"),
            record["image_url"].as_str().unwrap(),
            record["processed"].as_bool().unwrap(),
            record["created_at"].as_str().unwrap(),
            record["updated_at"].as_str().unwrap()
        );
        writeln!(sql_file, "{}", insert_sql)?;
    }
    
    info!("Created JSON backup: {}", backup_file);
    info!("Created SQL backup: {}", sql_backup_file);
    
    Ok(backup_file)
}

async fn wipe_metadata_table(pool: &PgPool) -> Result<u64> {
    // Count records before deletion
    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metadata")
        .fetch_one(pool)
        .await?;
    
    info!("Records before deletion: {}", count_before);
    
    // Delete all records
    let result = sqlx::query("DELETE FROM metadata")
        .execute(pool)
        .await?;
    
    let deleted_count = result.rows_affected();
    
    // Verify deletion
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM metadata")
        .fetch_one(pool)
        .await?;
    
    info!("Records after deletion: {}", count_after);
    
    if count_after != 0 {
        error!("Warning: {} records still remain in metadata table", count_after);
    }
    
    Ok(deleted_count)
}
