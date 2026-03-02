use slint::Weak;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{App, TOKIO_RUNTIME, app_state::AppState};

mod events;

pub use self::events::*;

#[derive(Debug)]
pub enum Commands {
    Events(EventCommands),
}

pub fn start_command_loop(mut rx: UnboundedReceiver<Commands>, state: AppState, app: Weak<App>) {
    TOKIO_RUNTIME.spawn(async move {
        while let Some(command) = rx.recv().await {
            if let Err(e) = command_handler(command, &state, app.clone()).await {
                tracing::error!(error = %e, "failed to perform command");
            }
        }

        tracing::info!("exiting tokio event loop");
    });
}

async fn command_handler(cmd: Commands, state: &AppState, app: Weak<App>) -> anyhow::Result<()> {
    match cmd {
        Commands::Events(cmd) => events::handle(cmd, state, app).await?,
    };

    Ok(())
}
