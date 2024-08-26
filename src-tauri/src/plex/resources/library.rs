use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Library {
    title: Arc<str>,
    key: Arc<str>,
    #[serde(rename = "type")]
    media_type: Arc<str>, // could be enum
}

impl Library {
    pub(crate) fn key_ref(&self) -> &str {
        self.key.as_ref()
    }

    pub(crate) fn into_key_val(self) -> (Arc<str>, Self) {
        (self.title.clone(), self)
    }

    pub(crate) fn title_ref(&self) -> &str {
        self.title.as_ref()
    }
}
