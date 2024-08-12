use derive_more::Display;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::plex::Plex;

use super::Error;

#[derive(Serialize, Deserialize, Display, Default)]
pub(crate) struct AppSettings {
    pub(crate) plex: Plex,
}

impl AppSettings {
    const STORE: &'static str = "settings";
    pub(super) fn from_store(store: &mut Store<Wry>) -> Self {
        debug!("Loading {} store", Self::STORE);
        let settings = if let Some(settings) = store.get(Self::STORE) {
            serde_json::from_value(settings.to_owned()).map_err(|err| err.into())
        } else {
            Err(Error::StoreEmpty)
        };

        if let Ok(settings) = settings {
            settings
        } else {
            warn!("Failed to find store for: {}", Self::STORE);
            let settings = Self::default();
            store
                .insert(
                    Self::STORE.to_string(),
                    serde_json::to_value(&settings).unwrap_or_default(),
                )
                .ok();
            store.save().ok(); // maybe could use settings.save(store)
            settings
        }
    }

    pub(super) fn save(&self, store: &mut Store<Wry>) {
        debug!("Saving to {} store", Self::STORE);
        store
            .insert(
                Self::STORE.to_string(),
                serde_json::to_value(self).unwrap_or_default(),
            )
            .ok();
        store.save().ok();
    }
}
