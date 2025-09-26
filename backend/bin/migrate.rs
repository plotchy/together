use together::{utils, get_db_pool};
use together::db::DatabaseConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::init_logging();
    
    let db_config = DatabaseConfig::from_env()?;
    let pool = get_db_pool(&db_config).await?;
    
    println!("Running database migrations...");
    together::db::migrations::run_migrations(&pool).await?;
    println!("Migrations completed successfully!");
    
    Ok(())
}
