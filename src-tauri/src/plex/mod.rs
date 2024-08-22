mod error;

pub use error::*;
use log::{debug, trace, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;

use std::{env, sync::Arc, time::Duration};

use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct PlexServer {
    name: Box<str>,
    uri: Arc<str>,
}
#[derive(Serialize, Deserialize, Display)]
#[display(fmt = "{}", "serde_json::to_string(self).unwrap()")]
pub(crate) struct Plex {
    client_ident: Box<str>,
    user_token: Option<Arc<str>>,
    selected_server: Option<PlexServer>,
    selected_library: Option<Library>,
    #[serde(skip_serializing)] // Im 50/50 on refreshing at startup
    resources: Option<Box<[PlexResource]>>, // TODO might be better as hashmap
    #[serde(skip_serializing)] // Im 50/50 on refreshing at startup
    libraries: Option<Box<[Library]>>, // TODO might be better as hashmap
    #[serde(skip_serializing)] // Im 50/50 on refreshing at startup
    albums: Option<Arc<[Album]>>, // TODO might be better as hashmap
    #[serde(skip_serializing)]
    #[serde(default = "default_session")]
    session_token: Box<str>,
    // TODO: generate new client on Deserialize
    // #[serde(skip_serializing)]
    // client: reqwest::Client,
    // TODO: figure out device info
    // #[serde(skip_serializing)]
    //device: Box<str>,
    // #[serde(skip_serializing)]
    //device_name: Box<str>,
}

fn default_session() -> Box<str> {
    Uuid::new_v4().to_string().into()
}

// TODO: Custom Deserializer for plex

impl Default for Plex {
    fn default() -> Self {
        Self {
            client_ident: Uuid::new_v4().to_string().into(), // I could probably just pass along the uuid itself
            user_token: None,
            selected_server: None,
            selected_library: None,
            libraries: None,
            resources: None,
            albums: None,
            session_token: default_session(),
        }
    }
}

impl Plex {
    fn create_client(&self) -> Result<reqwest::blocking::Client> {
        debug!("Creating plex client with default headers");
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("X-Plex-Provides", HeaderValue::from_static("player"));
        headers.insert("X-Plex-Platform", HeaderValue::from_static(env::consts::OS));
        headers.insert(
            "X-Plex-Platform-Version",
            HeaderValue::from_static(env::consts::ARCH),
        );
        headers.insert(
            "X-Plex-Client-Name",
            HeaderValue::from_static(env!("CARGO_PKG_NAME")),
        );
        headers.insert(
            "X-Plex-Client-Identifier",
            HeaderValue::from_str(self.client_ident.as_ref())?,
        );
        headers.insert(
            "X-Plex-Version",
            HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
        );
        headers.insert(
            "X-Plex-Product",
            HeaderValue::from_static(env!("CARGO_PKG_NAME")),
        );
        headers.insert(
            "X-Plex-Session-Identifier",
            HeaderValue::from_str(self.session_token.as_ref())?,
        );
        // headers.insert("X-Plex-Device",
        //     HeaderValue::from_str(self.device.as_ref())?,

        // );
        // headers.insert("X-Plex-Device-Name",
        //     HeaderValue::from_str(self.device_name.as_ref())?,
        // );
        if let Some(token) = &self.user_token {
            headers.insert("X-Plex-Token", HeaderValue::from_str(token.as_ref())?);
        }

        Ok(reqwest::blocking::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(5))
            .build()?)
    }

    pub(crate) fn create_login_pin(&self) -> Result<PlexPin> {
        debug!("Generating login pin");
        let client = self.create_client()?; // TODO: shared client in state

        let pin = PlexPin::new(&client)?;

        debug!("Created pin {:#?}", pin);

        Ok(pin)
    }

    pub(crate) fn check_pin(&mut self, pin: &PlexPin) -> Result<()> {
        debug!("Checking login pin status");
        let client = self.create_client()?; // TODO: shared client in state
        let checked_pin = pin.check_pin(&client)?;
        self.user_token = checked_pin.auth_token;
        if self.user_token.is_some() {
            self.refresh_all_unchecked();
            Ok(())
        } else {
            Err(Error::WaitingOnPin)
        }
    }

    pub(crate) fn has_user(&self) -> bool {
        self.user_token.is_some()
    }

    pub(crate) fn signout(&mut self) {
        debug!("Removing plex");
        // Is there a plex call to deauth a token?
        self.user_token = None;
        self.selected_server = None;
        self.selected_library = None;
        self.libraries = None;
        self.resources = None;
        self.albums = None;
        self.session_token = default_session();
    }

    pub(crate) fn refresh_all_unchecked(&mut self) {
        debug!("refreshing all plex data");
        self.refresh_resources().ok();
        self.refresh_libraries().ok();
        self.refresh_albums().ok();
    }

    pub(crate) fn refresh_resources(&mut self) -> Result<()> {
        debug!("refreshing resources");
        if self.user_token.is_none() {
            return Err(Error::NotAuthenticated);
        }
        let client = self.create_client()?; // TODO: shared client in state
        let resources = PlexResource::list(&client)?;
        self.resources = Some(resources);
        Ok(())
    }

    pub(crate) fn refresh_libraries(&mut self) -> Result<()> {
        debug!("refreshing libraries");
        if self.user_token.is_none() {
            return Err(Error::NotAuthenticated);
        }
        let server = self
            .selected_server
            .as_ref()
            .ok_or(Error::NoServerSelected)?;
        let client = self.create_client()?; // TODO: shared client in state
        self.libraries = Some(Library::list(&client, server.uri.as_ref())?);
        Ok(())
    }

    pub(crate) fn refresh_albums(&mut self) -> Result<()> {
        debug!("refreshing albums");
        if self.user_token.is_none() {
            return Err(Error::NotAuthenticated);
        }
        let server = self
            .selected_server
            .as_ref()
            .ok_or(Error::NoServerSelected)?;
        let library = self
            .selected_library
            .as_ref()
            .ok_or(Error::NoLibrarySelected)?;
        let client = self.create_client()?; // TODO: shared client in state
        let auth = self.user_token.as_ref().ok_or(Error::NotAuthenticated)?;
        let token = format!("?X-Plex-Token={auth}");
        self.albums = Some(library.all(&client, &server.uri, &token)?);
        debug!("found {} albums", self.albums.as_ref().unwrap().len());
        Ok(())
    }

    pub(crate) fn get_servers(&self) -> Result<Box<[&str]>> {
        debug!("listing resources");
        let resources = self.resources.as_ref().ok_or(Error::NoResourcesFound)?;

        //let client = self.create_client()?; // TODO: shared client in state
        Ok(resources
            .par_iter()
            //.filter(|res| res.get_first_working_connection(&client).is_ok())
            .map(|resource| resource.name.as_ref())
            .collect())
    }

    pub(crate) fn select_server(&mut self, server: &str) -> Result<()> {
        debug!("selecting resource");
        if self.user_token.is_none() {
            return Err(Error::NotAuthenticated);
        }
        let resources = self.resources.as_ref().ok_or(Error::NoResourcesFound)?;
        let client = self.create_client()?; // TODO: shared client in state
        let server = resources
            .par_iter()
            .find_any(|res| res.name == server.into())
            .ok_or(Error::InvalidSeverName)?
            .get_first_working_connection(&client)
            .map(|conn| PlexServer {
                name: server.into(),
                uri: conn.uri.clone(),
            })?;

        self.selected_server = Some(server);

        self.refresh_libraries()?;

        Ok(())
    }

    pub(crate) fn get_selected_server(&self) -> Option<&str> {
        self.selected_server
            .as_ref()
            .map(|server| server.name.as_ref())
    }

    pub(crate) fn reset_server_selection(&mut self) {
        debug!("reseting selected resource");
        self.selected_server = None;
    }

    pub(crate) fn get_libraries(&self) -> Result<Box<[&str]>> {
        let libraries = self.libraries.as_ref().ok_or(Error::NoLibrariesFound)?;

        Ok(libraries
            .as_ref()
            .par_iter()
            .map(|lib| lib.title.as_ref())
            .collect()) // maybe filter by music category
    }

    pub(crate) fn select_library(&mut self, server: &str) -> Result<()> {
        debug!("selecting library");
        let libraries = self.libraries.as_ref().ok_or(Error::NoLibrariesFound)?;

        let library = libraries
            .par_iter()
            .find_any(|res| res.title == server.into())
            .ok_or(Error::InvalidLibraryName)?;

        self.selected_library = Some(library.clone());

        self.refresh_albums()?;

        Ok(())
    }

    pub(crate) fn get_selected_library(&self) -> Option<&str> {
        self.selected_library
            .as_ref()
            .map(|library| library.title.as_ref())
    }

    pub(crate) fn reset_library_selection(&mut self) {
        debug!("reseting selected librarty");
        self.selected_library = None;
    }

    pub(crate) fn get_albums(&self) -> Result<&[Album]> {
        debug!("get albums");
        self.albums
            .as_ref()
            .ok_or(Error::NoAlbumsFound)
            .map(|album| album.as_ref())
    }

    pub(crate) fn get_album(&self, key: &str) -> Result<Album> {
        debug!("get album: {key}");
        Ok(self
            .albums
            .as_ref()
            .ok_or(Error::NoAlbumsFound)?
            .par_iter()
            .find_any(|album| album.rating_key.as_ref() == key)
            .ok_or(Error::NoAlbumFound)?
            .clone())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlexPin {
    id: u64,
    code: Arc<str>,
    auth_token: Option<Arc<str>>,
}

impl PlexPin {
    fn new(client: &reqwest::blocking::Client) -> Result<Self> {
        let pin_url = "https://plex.tv/api/v2/pins";
        Ok(client.post(pin_url).send()?.json()?)
    }

    fn check_pin(&self, client: &reqwest::blocking::Client) -> Result<Self> {
        let pin_url = format!("https://plex.tv/api/v2/pins/{}", self.id);

        trace!("Checking pin {:#?}", self);
        let res = client.get(pin_url).send()?;

        Ok(res.json()?)
    }

    pub(crate) fn get_pin(&self) -> &str {
        self.code.as_ref()
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlexConnections {
    uri: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlexResource {
    name: Box<str>,
    connections: Box<[PlexConnections]>,
}

impl PlexResource {
    fn list(client: &reqwest::blocking::Client) -> Result<Box<[Self]>> {
        let url = "https://plex.tv/api/v2/resources";
        Ok(client.get(url).send()?.json()?)
    }

    fn get_first_working_connection(
        &self,
        client: &reqwest::blocking::Client,
    ) -> Result<&PlexConnections> {
        self.connections
            .par_iter()
            .find_any(|conn| client.get(conn.uri.as_ref()).send().is_ok())
            .ok_or(Error::NoValidConnections)
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Library {
    title: Arc<str>,
    key: Arc<str>,
    #[serde(rename = "type")]
    media_type: Arc<str>, // could be enum
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Album {
    pub(crate) title: Arc<str>,
    pub(crate) rating_key: Arc<str>,
    pub(crate) summary: Arc<str>,
    studio: Option<Arc<str>>,
    #[serde(default)]
    pub(crate) thumb: Arc<str>,
    pub(crate) parent_title: Arc<str>,
    parent_rating_key: Arc<str>,
    pub(crate) year: Option<u64>,
    index: u64,
}

impl Library {
    fn list(client: &reqwest::blocking::Client, uri: &str) -> Result<Box<[Self]>> {
        let uri = format!("{uri}/library/sections/");
        debug!("retrieving libraries from: {uri}");
        Ok(serde_json::from_value(
            client
                .get(uri)
                .send()?
                .json::<Value>()?
                .get("MediaContainer")
                .ok_or(Error::MediaContainerNotFound)?
                .get("Directory")
                .ok_or(Error::LibraryDirectoryNotFound)?
                .to_owned(),
        )?)
    }

    fn all(
        &self,
        client: &reqwest::blocking::Client,
        base_uri: &str,
        token: &str,
    ) -> Result<Arc<[Album]>> {
        let key = self.key.as_ref();
        let uri = format!("{base_uri}/library/sections/{key}/all");

        debug!("retrieving albums from: {uri}");
        let res = client
            .get(uri)
            .query(&[("type", "9")]) // only retrieve albums
            .send()?
            .json::<Value>()?
            .get("MediaContainer")
            .ok_or(Error::MediaContainerNotFound)?
            .get("Metadata")
            .ok_or(Error::LibraryMetadataNotFound)?
            .to_owned();

        match serde_json::from_value::<Arc<[Album]>>(res) {
            Ok(v) => Ok(v
                .par_iter()
                .map(|album| {
                    let mut album = album.clone();
                    let thumb = album.thumb.as_ref();
                    album.thumb = format!("{base_uri}{thumb}{token}").into();

                    album
                })
                .collect()),
            Err(e) => {
                warn!("{e:#?}");
                Err(e.into())
            }
        }
    }
}
