mod core;
mod database;
use std::{
    path::PathBuf,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use slint::{Image, ModelRc, ToSharedString, VecModel};

use crate::database::events::RawEvent;

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

    let ui = App::new()?;

    TOKIO_RUNTIME.block_on({
        let ui = ui.clone_strong();

        async move {
            let mut connection = db_pool.acquire().await.unwrap();

            let db_events = database::events::events(&mut connection, 0, 100)
                .await
                .unwrap();

            let events: Vec<EventData> = db_events
                .into_iter()
                .map(
                    |RawEvent {
                         id, name, svg_icon, ..
                     }| EventData {
                        icon: Image::load_from_svg_data(
                            &data_encoding::BASE64.decode(svg_icon.as_bytes()).unwrap(),
                        )
                        .unwrap(),
                        id,
                        name: name.to_shared_string(),
                    },
                )
                .collect();

            ui.set_events(ModelRc::new(Rc::new(VecModel::from(events))));
        }
    });

    ui.run()?;

    Ok(())
}
