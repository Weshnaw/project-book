use derive_more::{Display, Error, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Error, Display)]
pub enum Error {
    StoreEmpty,
    InvalidJson(serde_json::Error),
    StoreFailed(tauri_plugin_store::Error),
    NoBookFound,
    BookNotDownloaded,
}
