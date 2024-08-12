use derive_more::Display;
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::plex::Plex;

#[derive(Serialize, Deserialize, Display, Default)]
pub(crate) struct AppSettings {
    pub(crate) plex: Plex,
}

impl AppSettings {
    const STORE: &'static str = "settings";
    pub(super) fn from_store(store: &mut Store<Wry>) -> Self {
        if let Some(settings) = store.get(Self::STORE) {
            serde_json::from_value(settings.to_owned()).unwrap() // TODO: handle gracefully
        } else {
            let settings = Self::default();
            store
                .insert(
                    Self::STORE.to_string(),
                    serde_json::to_value(&settings).unwrap_or_default(),
                )
                .ok();
            store.save().ok();

            settings
        }
    }

    pub(super) fn save(&self, store: &mut Store<Wry>) {
        store
            .insert(
                Self::STORE.to_string(),
                serde_json::to_value(self).unwrap_or_default(),
            )
            .ok();
        store.save().ok();
    }
}
