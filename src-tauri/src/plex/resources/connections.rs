use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlexResource {
    name: Arc<str>,
    connections: Box<[PlexConnections]>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlexConnections {
    uri: Arc<str>,
}

impl PlexResource {
    pub(crate) fn into_key_val(self) -> (Arc<str>, Self) {
        (self.name.clone(), self)
    }
    pub(crate) fn connections_ref(&self) -> &[PlexConnections] {
        self.connections.as_ref()
    }
}

impl PlexConnections {
    pub(crate) fn clone_uri(&self) -> Arc<str> {
        self.uri.clone()
    }
    pub(crate) fn uri_ref(&self) -> &str {
        self.uri.as_ref()
    }
}
