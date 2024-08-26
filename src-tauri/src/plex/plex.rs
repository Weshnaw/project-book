use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
    time::Duration,
};

use derive_more::Display;
use log::debug;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    client::BoxedClient,
    resources::{Album, Library, PlexResource},
    Error, Result,
};

#[derive(Serialize, Deserialize)]
struct SelectedConnection {
    name: Box<str>,
    uri: Arc<str>,
}

#[derive(Serialize, Deserialize)]
pub struct PlexData {
    client_ident: Box<str>,
    user_token: Option<Arc<str>>,
    selected_connection: Option<SelectedConnection>,
    selected_library: Option<Library>,

    // Generated new uuid if doesnt exist
    #[serde(skip_serializing)]
    #[serde(default = "default_session")]
    session_token: Box<str>,
    // TODO: figure out device info
    // #[serde(skip_serializing)]
    //device: Box<str>,
    // #[serde(skip_serializing)]
    //device_name: Box<str>,
}

fn default_session() -> Box<str> {
    Uuid::new_v4().to_string().into()
}

#[derive(Display)]
#[display(fmt = "{}", "serde_json::to_string(&self.data).unwrap()")]
pub(crate) struct Plex {
    data: PlexData,
    client: Arc<RwLock<BoxedClient>>,

    // Generated on initialization, may move these to a local store for caching
    resources: HashMap<Arc<str>, PlexResource>,
    libraries: HashMap<Arc<str>, Library>,
    albums: Arc<HashMap<Arc<str>, Album>>,
}

impl Plex {
    fn refresh(&mut self) -> Result<()> {
        let client = self.client.read()?;
        self.resources = self.data.get_resources(&client).unwrap_or_default();
        self.libraries = self.data.get_libraries(&client).unwrap_or_default();
        self.albums = Arc::new(self.data.get_albums(&client).unwrap_or_default());

        Ok(())
    }

    pub(crate) fn create_login_pin(&self) -> Result<PlexPin> {
        debug!("Generating login pin");

        let client = self.client.read()?;
        let pin = client.generate_pin()?;

        debug!("Created pin {:#?}", pin);

        Ok(pin)
    }

    fn refresh_client(&self) -> Result<()> {
        let mut client = self.client.write()?;
        *client = self.data.create_client()?;
        Ok(())
    }

    pub(crate) fn check_pin(&mut self, pin: &PlexPin) -> Result<()> {
        debug!("Checking login pin status");
        let client = self.client.read()?;
        let checked_pin = client.check_pin(pin.id)?;
        self.data.user_token = checked_pin.auth_token;
        drop(client);
        if self.data.user_token.is_some() {
            self.refresh_client()?;
            self.refresh()?;
            Ok(())
        } else {
            Err(Error::WaitingOnPin)
        }
    }

    pub(crate) fn get_servers(&self) -> Box<[&str]> {
        debug!("listing resources");
        self.resources
            .par_iter()
            .map(|(key, _)| key.as_ref())
            .collect()
    }

    pub(crate) fn select_server(&mut self, server: &str) -> Result<()> {
        debug!("selecting resource");
        let resource = self.resources.get(server).ok_or(Error::InvalidSeverName)?;

        let client = self.client.read()?;

        let selected = client
            .find_working_connection(resource)
            .map(|conn| SelectedConnection {
                name: server.into(),
                uri: conn.clone_uri(),
            })?;

        self.data.selected_connection = Some(selected);

        self.libraries = self.data.get_libraries(&client).unwrap_or_default();
        self.albums = Arc::new(self.data.get_albums(&client).unwrap_or_default());

        Ok(())
    }
    pub(crate) fn get_libraries(&self) -> Box<[&str]> {
        self.libraries
            .par_iter()
            .map(|(key, _)| key.as_ref())
            .collect()
    }

    pub(crate) fn select_library(&mut self, server: &str) -> Result<()> {
        debug!("selecting library");
        let library = self
            .libraries
            .get(server)
            .ok_or(Error::InvalidLibraryName)?;

        self.data.selected_library = Some(library.clone());

        let client = self.client.read()?;
        self.albums = Arc::new(self.data.get_albums(&client).unwrap_or_default());

        Ok(())
    }
    pub(crate) fn get_albums(&self) -> Result<Box<[&Album]>> {
        debug!("get albums");

        Ok(self.albums.values().collect())
    }

    pub(crate) fn get_album(&self, key: &str) -> Result<&Album> {
        debug!("get album: {key}");

        self.albums.get(key).ok_or(Error::NoAlbumFound)
    }

    pub(crate) fn signout(&mut self) -> Result<()> {
        debug!("Removing plex");

        self.data.signout();
        self.refresh_client()?;

        Ok(())
    }

    pub(crate) fn has_user(&self) -> bool {
        self.data.user_token.is_some()
    }

    pub(crate) fn authenticated_thumb(&self, thumb: &str) -> Result<String> {
        let base_uri = self
            .data
            .selected_connection
            .as_ref()
            .ok_or(Error::NoServerSelected)?
            .uri
            .as_ref();
        let token = self
            .data
            .user_token
            .as_ref()
            .ok_or(Error::NotAuthenticated)?;
        let token = format!("?X-Plex-Token={token}");
        Ok(format!("{base_uri}{thumb}{token}"))
    }

    pub(crate) fn get_selected_server(&self) -> Option<&str> {
        self.data
            .selected_connection
            .as_ref()
            .map(|server| server.name.as_ref())
    }

    pub(crate) fn reset_server_selection(&mut self) {
        debug!("reseting selected resource");
        self.data.selected_connection = None;
    }

    pub(crate) fn get_selected_library(&self) -> Option<&str> {
        self.data
            .selected_library
            .as_ref()
            .map(|library| library.title_ref())
    }

    pub(crate) fn reset_library_selection(&mut self) {
        debug!("reseting selected librarty");
        self.data.selected_library = None;
    }
}

impl From<PlexData> for Plex {
    fn from(data: PlexData) -> Self {
        let client = data.create_client().unwrap();
        let (resources, libraries, albums) = if data.user_token.is_some() {
            let resources = data.get_resources(&client).unwrap_or_default();
            let libraries = data.get_libraries(&client).unwrap_or_default();
            let albums = data.get_albums(&client).unwrap_or_default();

            (resources, libraries, Arc::new(albums))
        } else {
            (HashMap::new(), HashMap::new(), Arc::new(HashMap::new()))
        };
        let client = Arc::new(RwLock::new(client));

        Self {
            data,
            client,
            resources,
            libraries,
            albums,
        }
    }
}

impl Default for Plex {
    fn default() -> Self {
        let data = PlexData::default();
        Self::from(data)
    }
}

impl Serialize for Plex {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Plex {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data: PlexData = PlexData::deserialize(deserializer)?;
        Ok(Self::from(data))
    }
}

impl Default for PlexData {
    fn default() -> Self {
        Self {
            client_ident: Uuid::new_v4().to_string().into(), // I could probably just pass along the uuid itself
            user_token: None,
            selected_connection: None,
            selected_library: None,
            session_token: default_session(),
        }
    }
}

impl PlexData {
    #[cfg(not(debug_assertions))]
    fn create_client(&self) -> Result<BoxedClient> {
        Ok(Box::new(self.__create_client()?))
    }

    #[cfg(test)]
    fn create_client(&self) -> Result<BoxedClient> {
        use super::client::mock::MockPlexClient;
        Ok(Box::new(MockPlexClient))
    }

    #[cfg(all(debug_assertions, not(test)))]
    fn create_client(&self) -> Result<BoxedClient> {
        use super::client::mock::MockPlexClient;

        let client: BoxedClient = match env::var("USE_MOCK_PLEX") {
            Ok(val) => match val.trim().to_lowercase().as_str() {
                "f" | "0" | "false" => Box::new(self.__create_client()?),
                _ => Box::new(MockPlexClient),
            },
            Err(_) => Box::new(self.__create_client()?),
        };

        Ok(client)
    }

    fn __create_client(&self) -> Result<reqwest::blocking::Client> {
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

    fn signout(&mut self) {
        debug!("Removing plex");
        // Is there a plex call to deauth a token?
        self.user_token = None;
        self.selected_connection = None;
        self.selected_library = None;
        self.session_token = default_session();
    }

    fn get_resources(&self, client: &BoxedClient) -> Result<HashMap<Arc<str>, PlexResource>> {
        debug!("refreshing resources");
        let resources = client.resources()?;

        Ok(resources
            .into_par_iter()
            .map(|res| res.into_key_val())
            .collect())
    }

    fn get_libraries(&self, client: &BoxedClient) -> Result<HashMap<Arc<str>, Library>> {
        debug!("refreshing libraries");
        let server = self
            .selected_connection
            .as_ref()
            .ok_or(Error::NoServerSelected)?;
        let libraries = client.libraries(&server.uri)?;

        Ok(libraries
            .into_par_iter()
            .map(|lib| lib.into_key_val())
            .collect())
    }

    fn get_albums(&self, client: &BoxedClient) -> Result<HashMap<Arc<str>, Album>> {
        debug!("refreshing albums");
        let server = self
            .selected_connection
            .as_ref()
            .ok_or(Error::NoServerSelected)?;
        let library = self
            .selected_library
            .as_ref()
            .ok_or(Error::NoLibrarySelected)?;

        let albums = client.albums(&server.uri, library.key_ref())?;
        debug!("found {} albums", albums.len());
        Ok(albums
            .into_par_iter()
            .map(|album| album.into_key_val())
            .collect())
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlexPin {
    id: u64,
    code: Arc<str>,
    auth_token: Option<Arc<str>>,
}

impl PlexPin {
    #[cfg(debug_assertions)]
    pub(crate) fn authed_pin() -> Self {
        Self {
            auth_token: Some("1234".into()),
            ..Default::default()
        }
    }

    pub(crate) fn pin_ref(&self) -> &str {
        self.code.as_ref()
    }
}
