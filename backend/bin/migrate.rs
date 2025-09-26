use dwrcasts::{utils, get_db_pool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::init_logging();
    
    let db_config = dwrcasts::db::DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    println!("Running database migrations...");
    dwrcasts::db::migrations::run_migrations(&pool).await?;
    println!("Migrations completed successfully!");
    
    Ok(())
}
