use slint::ComponentHandle;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    core::{Commands, EventCommands},
    *,
};

pub fn handle_commands(app: App, app_tx: UnboundedSender<Commands>) {
    app.global::<AppLogic>().on_request_events({
        let app_tx = app_tx.clone();

        move || {
            let _ = app_tx.send(Commands::Events(EventCommands::List(0)));
        }
    });

    app.global::<AppLogic>().on_event_clicked({
        let app_tx = app_tx.clone();

        move |id| {
            let _ = app_tx.send(Commands::Events(EventCommands::Clicked(id as u32)));
        }
    });
}
