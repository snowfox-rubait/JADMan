use sqlx::{SqlitePool, Result};

pub async fn init_db(pool: &SqlitePool) -> Result<()> {
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
        
    Ok(())
}
