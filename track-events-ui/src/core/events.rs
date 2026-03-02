use std::rc::Rc;

use slint::{Image, ModelRc, ToSharedString, VecModel, Weak};

use crate::{
    App, EventData,
    app_state::AppState,
    database::{self, events::RawEvent},
};

const ELEMENTS_LIMIT: usize = 100;

#[derive(Debug)]
pub enum EventCommands {
    List(u32),
}

impl TryFrom<RawEvent> for EventData {
    type Error = anyhow::Error;

    fn try_from(
        RawEvent {
            id, name, svg_icon, ..
        }: RawEvent,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            icon: Image::load_from_svg_data(&data_encoding::BASE64.decode(svg_icon.as_bytes())?)?,
            id,
            name: name.to_shared_string(),
        })
    }
}

pub(super) async fn handle(
    cmd: EventCommands,
    state: &AppState,
    app: Weak<App>,
) -> anyhow::Result<()> {
    match cmd {
        EventCommands::List(offset) => list(&state.pool, app, offset).await,
    }
}

pub async fn list(pool: &sqlx::SqlitePool, app: Weak<App>, offset: u32) -> anyhow::Result<()> {
    let connection = pool.acquire().await?;

    let events = database::events::events(connection, offset, ELEMENTS_LIMIT).await?;

    let _ = app.upgrade_in_event_loop(move |app| {
        // You want believe, but EventData is not sync and you need to perform map on ui thread
        let events = events
            .into_iter()
            .filter_map(|this| TryFrom::try_from(this).ok())
            .collect::<Vec<EventData>>();

        let _ = app.set_events(ModelRc::new(Rc::new(VecModel::from(events))));
    });

    Ok(())
}
