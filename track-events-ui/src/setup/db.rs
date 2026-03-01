use std::path::Path;

use anyhow::Context;

const APPLY_MIGRATIONS: bool = true;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../sql/");

pub async fn setup_db(
    database_location: impl AsRef<Path>,
) -> anyhow::Result<sqlx::Pool<sqlx::Sqlite>> {
    tracing::debug!(
        "Database location: {}",
        database_location.as_ref().to_string_lossy()
    );

    let db_pool = connect(&database_location, APPLY_MIGRATIONS).await;

    if let Err(_) = db_pool {
        tokio::fs::remove_file(database_location.as_ref())
            .await
            .context("db deletion")?;
        tracing::debug!("Database deleted. Creating new");
        connect(database_location, APPLY_MIGRATIONS).await
    } else {
        db_pool
    }
}

async fn connect(
    database_location: impl AsRef<Path>,
    migrate: bool,
) -> anyhow::Result<sqlx::SqlitePool> {
    let opt = sqlx::sqlite::SqliteConnectOptions::new()
        .create_if_missing(true)
        .read_only(false)
        .filename(database_location);

    let pool = sqlx::SqlitePool::connect_with(opt).await?;

    if migrate {
        MIGRATOR.run(&pool).await?;
    }

    Ok(pool)
}
