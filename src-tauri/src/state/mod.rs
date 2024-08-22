mod books;
mod error;
mod settings;

pub use error::*;

pub(crate) use books::*;
use log::info;
pub(crate) use settings::*;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tauri::{App, Manager, Wry};
use tauri_plugin_store::{Store, StoreBuilder};

use crate::plex::PlexPin;

pub(crate) type AppState = Mutex<InnerAppState>;
pub(crate) struct InnerAppState {
    pub(crate) settings: AppSettings,
    pub(crate) current_book: Option<Arc<str>>, // could potentially just be the key
    pub(crate) store: Store<Wry>,
    pub(crate) plex_pin: Option<PlexPin>,
    pub(crate) books: HashMap<Arc<str>, Book>,
}

impl InnerAppState {
    pub(crate) fn save_settings(&mut self) {
        self.settings.save(&mut self.store)
    }

    pub(crate) fn save_books(&mut self) {
        self.books.save(&mut self.store).ok();
    }

    pub(crate) fn save_book(&mut self, key: &str) {
        if let Some(book) = self.books.get(key) {
            book.save(&mut self.store).ok();
        }
    }

    pub(crate) fn save_current_book(&mut self) {
        self.store
            .insert(
                Book::CURRENT_BOOK_STORE.into(),
                serde_json::to_value(self.current_book.clone()).unwrap_or_default(),
            )
            .ok();
        self.store.save().ok();
    }
}

pub(crate) const BIN: &str = "store.bin";

pub(crate) fn setup_state(app: &mut App) -> core::result::Result<(), Box<dyn std::error::Error>> {
    info!("Loading stored data");
    let mut store = StoreBuilder::new(BIN).build(app.handle().clone());

    store.load().ok();
    let mut settings = AppSettings::from_store(&mut store);
    let current_book = None; // Book::get_current(&store); // TODO create player on startup
    let books = Book::get_all_books(&mut store);

    settings.plex.refresh_all_unchecked(); // Im 50/50 on refreshing at startup

    app.manage(Mutex::new(InnerAppState {
        settings,
        current_book,
        store,
        books,
        plex_pin: None,
    }));

    Ok(())
}
