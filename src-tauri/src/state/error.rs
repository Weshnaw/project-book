use derive_more::{Display, Error, From};

use crate::plex;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {
    StoreEmpty,
    InvalidJson(serde_json::Error),
    StoreFailed(tauri_plugin_store::Error),
    Plex(plex::Error),
    NoBookFound,
    BookNotDownloaded,
}
