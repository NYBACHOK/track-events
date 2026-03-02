use tokio::sync::mpsc::UnboundedSender;

use crate::{
    App,
    core::{Commands, EventCommands},
};

pub fn handle_commands(app: App, app_tx: UnboundedSender<Commands>) {
    app.on_request_events({
        let app_tx = app_tx.clone();

        move || {
            let _ = app_tx.send(Commands::Events(EventCommands::List(0)));
        }
    });
}
