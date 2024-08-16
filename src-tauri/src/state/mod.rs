mod books;
mod error;
mod settings;

pub use error::*;

pub(crate) use books::*;
use log::info;
pub(crate) use settings::*;

use std::sync::Mutex;

use tauri::{App, Manager, Wry};
use tauri_plugin_store::{Store, StoreBuilder};

use crate::plex::PlexPin;

pub(crate) type AppState = Mutex<InnerAppState>;
pub(crate) struct InnerAppState {
    pub(crate) settings: AppSettings,
    pub(crate) books: Books,
    pub(crate) store: Store<Wry>,
    pub(crate) plex_pin: Option<PlexPin>,
}

impl InnerAppState {
    pub(crate) fn save_settings(&mut self) {
        self.settings.save(&mut self.store)
    }
}

pub(crate) fn setup_state(app: &mut App) -> core::result::Result<(), Box<dyn std::error::Error>> {
    info!("Loading stored data");
    let mut store = StoreBuilder::new("store.bin").build(app.handle().clone());

    store.load().ok();
    let mut settings = AppSettings::from_store(&mut store);
    let books = Books::from_store(&mut store);

    settings.plex.refresh_all_unchecked(); // Im 50/50 on refreshing at startup

    app.manage(Mutex::new(InnerAppState {
        settings,
        books,
        store,
        plex_pin: None,
    }));

    Ok(())
}
