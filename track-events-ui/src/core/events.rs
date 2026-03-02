use slint::{Image, ToSharedString};

use crate::{
    EventData,
    app_state::AppState,
    database::{self, events::RawEvent},
};

#[derive(Debug)]
pub enum EventCommands {
    List(tokio::sync::oneshot::Sender<Vec<Event>>),
}

#[derive(Debug)]
pub struct Event {
    pub id: i32,
    pub icon: Vec<u8>,
    pub name: String,
}

impl From<Event> for EventData {
    fn from(Event { id, icon, name }: Event) -> Self {
        Self {
            icon: Image::load_from_svg_data(&icon).expect("always valid"),
            id,
            name: name.to_shared_string(),
        }
    }
}

impl From<RawEvent> for Event {
    fn from(
        RawEvent {
            id, name, svg_icon, ..
        }: RawEvent,
    ) -> Self {
        Self {
            id,
            icon: data_encoding::BASE64
                .decode(svg_icon.as_bytes())
                .expect("always valid"),
            name,
        }
    }
}

pub(super) async fn handle(cmd: EventCommands, state: &AppState) -> anyhow::Result<()> {
    match cmd {
        EventCommands::List(callback) => {
            let _ = callback.send(list(&state.pool).await?);
        }
    };

    Ok(())
}

async fn list(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<Event>> {
    let connection = pool.acquire().await?;

    let events = database::events::events(connection, 0, 100)
        .await?
        .into_iter()
        .map(From::from)
        .collect();

    Ok(events)
}
