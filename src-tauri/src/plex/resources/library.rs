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

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Album {
    title: Arc<str>,
    rating_key: Arc<str>,
    summary: Arc<str>,
    studio: Option<Arc<str>>,
    thumb: Option<Arc<str>>,
    parent_title: Option<Arc<str>>,
    parent_rating_key: Option<Arc<str>>,
    year: Option<u64>,
    index: u64,
}

impl Album {
    pub(crate) fn into_key_val(self) -> (Arc<str>, Self) {
        (self.rating_key.clone(), self)
    }

    pub(crate) fn key_ref(&self) -> &str {
        self.rating_key.as_ref()
    }

    pub(crate) fn parent_ref(&self) -> &str {
        self.parent_title
            .as_ref()
            .map(|val| val.as_ref())
            .unwrap_or_default()
    }

    pub(crate) fn summary_ref(&self) -> &str {
        self.summary.as_ref()
    }

    pub(crate) fn title_ref(&self) -> &str {
        self.title.as_ref()
    }

    pub(crate) fn thumb_ref(&self) -> &str {
        self.thumb
            .as_ref()
            .map(|val| val.as_ref())
            .unwrap_or_default()
    }

    pub(crate) fn key_clone(&self) -> Arc<str> {
        self.rating_key.clone()
    }
}
