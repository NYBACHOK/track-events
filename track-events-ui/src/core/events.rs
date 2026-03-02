use std::rc::Rc;

use slint::{Image, Model, ModelRc, ToSharedString, VecModel, Weak};

use crate::{
    App, EventData,
    app_state::AppState,
    database::{self, events::RawEvent},
};

const ELEMENTS_LIMIT: usize = 100;

#[derive(Debug)]
pub enum EventCommands {
    List(u32),
    Clicked(u32),
}

impl TryFrom<RawEvent> for EventData {
    type Error = anyhow::Error;

    fn try_from(
        RawEvent {
            id,
            name,
            svg_icon,
            event_occurrence,
            ..
        }: RawEvent,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            icon: Image::load_from_svg_data(&data_encoding::BASE64.decode(svg_icon.as_bytes())?)?,
            id,
            name: name.to_shared_string(),
            occurrence_count: event_occurrence,
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
        EventCommands::Clicked(id) => clicked(&state.pool, app, id).await,
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

pub async fn clicked(pool: &sqlx::SqlitePool, app: Weak<App>, id: u32) -> anyhow::Result<()> {
    database::events::event_occurrence_create(pool.acquire().await?, id).await?;

    let _ = app.upgrade_in_event_loop(move |app| {
        let model = app.get_events();

        for (i, mut event) in model.iter().enumerate() {
            if event.id as u32 == id {
                event.occurrence_count += 1;
                model.set_row_data(i, event);
                break;
            }
        }
    });

    Ok(())
}
