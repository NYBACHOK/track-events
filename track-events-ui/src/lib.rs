mod app_state;
mod core;
mod database;
use std::{path::PathBuf, sync::LazyLock};

use crate::app_state::AppState;

mod commands;
mod setup;

slint::include_modules!();

static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .inspect_err(|e| tracing::error!(error = %e, "tokio runtime initialization"))
        .expect("failed to init tokio runtime")
});

static APP_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    const BUNDLE_ID: &str = "com.track.events.application";

    dirs::data_dir()
        .unwrap_or_else(|| {
            let dir = std::env::current_dir().unwrap_or_default();

            tracing::error!(data_dir = %dir.display(), "failed to get data dir will use current dir");

            dir
        })
        .join(BUNDLE_ID)
});

pub fn start() -> anyhow::Result<()> {
    let db_pool = TOKIO_RUNTIME.block_on(setup::pre_start_setup())?;

    let app_state = AppState::new(db_pool);

    let app = App::new()?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    commands::handle_commands(app.clone_strong(), tx);

    core::start_command_loop(rx, app_state);

    tracing::info!("starting app");

    app.invoke_request_events();

    app.run()?;

    Ok(())
}
