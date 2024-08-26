use log::{debug, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;

use super::{
    resources::{
        connections::{PlexConnections, PlexResource},
        library::{AlbumData, Library},
    },
    Error, PlexPin, Result,
};

pub(super) type BoxedClient = Box<dyn PlexClient + Sync + Send>;

pub(super) trait PlexClient {
    fn find_working_connection<'b>(
        &self,
        resource: &'b PlexResource,
    ) -> Result<&'b PlexConnections>;
    fn albums(&self, key: &str, uri: &str) -> Result<Vec<AlbumData>>;
    fn libraries(&self, uri: &str) -> Result<Vec<Library>>;
    fn resources(&self) -> Result<Vec<PlexResource>>;
    fn check_pin(&self, id: u64) -> Result<PlexPin>;
    fn generate_pin(&self) -> Result<PlexPin>;
}

impl PlexClient for reqwest::blocking::Client {
    fn generate_pin(&self) -> Result<PlexPin> {
        let uri = "https://plex.tv/api/v2/pins";
        debug!("Generating pin using {uri}");
        Ok(self.post(uri).send()?.json()?)
    }

    fn check_pin(&self, id: u64) -> Result<PlexPin> {
        let uri = format!("https://plex.tv/api/v2/pins/{}", id);
        debug!("Checking pin using {uri}");
        Ok(self.get(uri).send()?.json()?)
    }

    fn resources(&self) -> Result<Vec<PlexResource>> {
        let uri = "https://plex.tv/api/v2/resources";
        debug!("Retrieving resources using {uri}");
        Ok(self.get(uri).send()?.json()?)
    }

    fn libraries(&self, uri: &str) -> Result<Vec<Library>> {
        let uri = format!("{uri}/library/sections/");
        debug!("Retrieving libraries using {uri}");
        Ok(serde_json::from_value(
            self.get(uri)
                .send()?
                .json::<Value>()?
                .get("MediaContainer")
                .ok_or(Error::MediaContainerNotFound)?
                .get("Directory")
                .ok_or(Error::LibraryDirectoryNotFound)?
                .to_owned(),
        )?)
    }

    fn albums(&self, uri: &str, key: &str) -> Result<Vec<AlbumData>> {
        let uri = format!("{uri}/library/sections/{key}/all");
        debug!("Retrieving albums using {uri}");

        let data = serde_json::from_value(
            self.get(uri)
                .query(&[("type", "9")]) // only retrieve albums
                .send()?
                .json::<Value>()?
                .get("MediaContainer")
                .ok_or(Error::MediaContainerNotFound)?
                .get("Metadata")
                .ok_or(Error::LibraryMetadataNotFound)?
                .to_owned(),
        );

        match data {
            Ok(v) => Ok(v),
            Err(e) => {
                warn!("Unable to retrieve albums: {:?}", e);
                Err(e.into())
            }
        }
    }

    fn find_working_connection<'b>(
        &self,
        resource: &'b PlexResource,
    ) -> Result<&'b PlexConnections> {
        resource
            .connections_ref()
            .par_iter()
            .find_any(|conn| self.get(conn.uri_ref()).send().is_ok())
            .ok_or(Error::NoValidConnections)
    }
}

#[cfg(debug_assertions)]
pub(crate) mod mock {
    use super::*;
    pub(crate) struct MockPlexClient;

    impl PlexClient for MockPlexClient {
        fn generate_pin(&self) -> Result<PlexPin> {
            Ok(PlexPin::default())
        }

        fn check_pin(&self, _id: u64) -> Result<PlexPin> {
            Ok(PlexPin::authed_pin())
        }

        fn resources(&self) -> Result<Vec<PlexResource>> {
            todo!()
        }

        fn libraries(&self, _uri: &str) -> Result<Vec<Library>> {
            todo!()
        }

        fn albums(&self, _key: &str, _uri: &str) -> Result<Vec<AlbumData>> {
            todo!()
        }

        fn find_working_connection<'b>(
            &self,
            _resource: &'b PlexResource,
        ) -> Result<&'b PlexConnections> {
            todo!()
        }
    }
}
