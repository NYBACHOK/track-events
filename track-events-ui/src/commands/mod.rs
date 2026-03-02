use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, VecModel};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    App, EventData,
    core::{Commands, EventCommands},
};

pub fn handle_commands(app: App, app_tx: UnboundedSender<Commands>) {
    app.on_request_events({
        let app = app.clone_strong();
        let app_tx = app_tx.clone();

        move || {
            let (tx, rx) = tokio::sync::oneshot::channel();

            let _ = app_tx.send(Commands::Events(EventCommands::List(tx)));

            let _ = slint::spawn_local({
                let app = app.clone_strong();

                async move {
                    let events = rx.await.unwrap();

                    let events: Vec<EventData> = events.into_iter().map(From::from).collect();

                    app.set_events(ModelRc::new(Rc::new(VecModel::from(events))));
                }
            });
        }
    });
}
