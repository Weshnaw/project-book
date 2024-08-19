use log::debug;
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::plex::Album;

use super::Error;

#[derive(Serialize, Deserialize, Clone)]
enum ReadingState {
    Playing,
    Paused,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Book {
    book_details: Album,
    state: ReadingState,
    progress: f64,
}

impl Book {
    pub(crate) fn new(album: Album) -> Self {
        Book {
            book_details: album,
            state: ReadingState::Playing,
            progress: 0f64,
        }
    }

    const STORE: &'static str = "current-book";
    pub(super) fn get_current(store: &mut Store<Wry>) -> Option<Self> {
        debug!("Loading {} store", Self::STORE);
        let book = if let Some(book) = store.get(Self::STORE) {
            serde_json::from_value(book.to_owned()).map_err(|err| err.into())
        } else {
            Err(Error::StoreEmpty)
        };

        book.ok()
    }
}
