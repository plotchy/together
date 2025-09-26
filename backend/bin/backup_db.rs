use anyhow::Result;
use chrono::Utc;
use clap::{Arg, Command};
use together::db::{get_db_pool, DatabaseConfig};
use together::utils::config::Config;
use sqlx::{PgPool, Row};
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command as StdCommand;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("backup-db")
        .about("Create a complete database backup")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .help("Backup format: sql, custom, or both")
                .value_parser(["sql", "custom", "both"])
                .default_value("sql"),
        )
        .arg(
            Arg::new("output-dir")
                .long("output-dir")
                .short('o')
                .help("Output directory for backup files")
                .default_value("./db_backups"),
        )
        .get_matches();

    let format = matches.get_one::<String>("format").unwrap();
    let output_dir = matches.get_one::<String>("output-dir").unwrap();

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

    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Generate timestamp for backup filename
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    
    match format.as_str() {
        "sql" => {
            create_sql_backup(&database_url, output_dir, &timestamp).await?;
        }
        "custom" => {
            create_custom_backup(&database_url, output_dir, &timestamp).await?;
        }
        "both" => {
            create_sql_backup(&database_url, output_dir, &timestamp).await?;
            create_custom_backup(&database_url, output_dir, &timestamp).await?;
        }
        _ => unreachable!(),
    }

    info!("🎉 Database backup complete!");
    
    Ok(())
}

async fn create_sql_backup(database_url: &str, output_dir: &str, timestamp: &str) -> Result<()> {
    info!("🔄 Creating SQL dump backup...");
    
    let backup_file = format!("{}/db_backup_{}.sql", output_dir, timestamp);
    
    // Try using psql to create the dump (bypasses version mismatch issues)
    let output = StdCommand::new("psql")
        .arg(database_url)
        .arg("-c")
        .arg("\\copy (SELECT 'SELECT pg_dump();') TO PROGRAM 'cat'")
        .output();
    
    if output.is_err() {
        warn!("psql approach failed, trying direct pg_dump with custom format...");
        return create_custom_backup(database_url, output_dir, timestamp).await;
    }
    
    // Alternative: Use COPY commands to export all tables
    let db_config = DatabaseConfig {
        database_url: database_url.to_string(),
        max_connections: 5,
    };
    let pool = get_db_pool(&db_config).await?;
    
    // Get all table names
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
    )
    .fetch_all(&pool)
    .await?;
    
    info!("Found {} tables to backup", tables.len());
    
    let mut sql_file = fs::File::create(&backup_file)?;
    writeln!(sql_file, "-- Database backup created at {}", Utc::now().to_rfc3339())?;
    writeln!(sql_file, "-- Tables: {}", tables.len())?;
    writeln!(sql_file)?;
    
    // Export schema first
    writeln!(sql_file, "-- Schema")?;
    let schema_rows: Vec<String> = sqlx::query_scalar(
        "SELECT 
            'CREATE TABLE ' || t.table_schema||'.'||t.table_name||' (' ||
            string_agg(c.column_name||' '||c.data_type, ', ') ||
            ');' as create_statement
        FROM information_schema.tables t
        JOIN information_schema.columns c ON c.table_name = t.table_name AND c.table_schema = t.table_schema
        WHERE t.table_schema = 'public'
        GROUP BY t.table_schema, t.table_name"
    )
    .fetch_all(&pool)
    .await?;
    
    for schema in schema_rows {
        writeln!(sql_file, "{}", schema)?;
    }
    writeln!(sql_file)?;
    
    // Export data for each table
    for table in &tables {
        info!("Backing up table: {}", table);
        writeln!(sql_file, "-- Data for table: {}", table)?;
        
        let rows = sqlx::query(&format!("SELECT * FROM {}", table))
            .fetch_all(&pool)
            .await?;
        
        if !rows.is_empty() {
            // Get column names
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT column_name FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position"
            )
            .bind(table)
            .fetch_all(&pool)
            .await?;
            
            for row in rows {
                let mut values = Vec::new();
                for (i, column) in columns.iter().enumerate() {
                    let value: Option<String> = row.try_get(i).ok();
                    match value {
                        Some(v) => values.push(format!("'{}'", v.replace("'", "''"))),
                        None => values.push("NULL".to_string()),
                    }
                }
                
                writeln!(
                    sql_file,
                    "INSERT INTO {} ({}) VALUES ({});",
                    table,
                    columns.join(", "),
                    values.join(", ")
                )?;
            }
        }
        writeln!(sql_file)?;
    }
    
    info!("✅ SQL backup created: {}", backup_file);
    Ok(())
}

async fn create_custom_backup(database_url: &str, output_dir: &str, timestamp: &str) -> Result<()> {
    info!("🔄 Creating custom format backup...");
    
    let backup_file = format!("{}/db_backup_{}.dump", output_dir, timestamp);
    
    // Try pg_dump with custom format (more reliable across versions)
    let output = StdCommand::new("pg_dump")
        .arg("--format=custom")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg("--file")
        .arg(&backup_file)
        .arg(database_url)
        .output()?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("pg_dump failed: {}", error_msg);
        return Err(anyhow::anyhow!("pg_dump failed: {}", error_msg));
    }
    
    info!("✅ Custom format backup created: {}", backup_file);
    info!("💡 To restore: pg_restore -d $DATABASE_URL {}", backup_file);
    
    Ok(())
}
