use db::*;

use crate::APP_DATA_DIR;

mod db;

pub async fn pre_start_setup() -> anyhow::Result<sqlx::Pool<sqlx::Sqlite>> {
    const DB_FILE_NAME: &str = ".event-track-db";

    setup_logger();

    let db_location = APP_DATA_DIR.join(DB_FILE_NAME);

    tokio::fs::create_dir_all(&*APP_DATA_DIR).await?;

    let db_pool = setup_db(db_location).await?;

    Ok(db_pool)
}

fn setup_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
                .add_directive("winit=warn".parse().unwrap())
                .add_directive("naga=warn".parse().unwrap())
                .add_directive("wgpu=warn".parse().unwrap())
                .add_directive("sctk=warn".parse().unwrap())
                .add_directive("hyper_util=warn".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap()),
        )
        .init();
}
