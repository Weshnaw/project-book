mod error;

pub use error::*;
use log::{debug, trace, warn};
use serde_json::Value;

use std::{env, sync::Arc, time::Duration};

use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
struct PlexServer {
    name: Arc<str>,
    uri: Arc<str>,
}

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
struct PlexLibrary {
    name: Arc<str>,
    key: Arc<str>,
}

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
pub(crate) struct Plex {
    client_ident: Arc<str>,
    user_token: Option<Arc<str>>,
    selected_server: Option<PlexServer>,
    selected_library: Option<PlexLibrary>,
    #[serde(skip_serializing)] // Im 50/50 on refreshing at startup
    resources: Option<Arc<[PlexResource]>>, // TODO might be better as hashmap
    #[serde(skip_serializing)] // Im 50/50 on refreshing at startup
    libraries: Option<Arc<[Library]>>, // TODO might be better as hashmap
    #[serde(skip_serializing)]
    #[serde(default = "default_session")]
    session_token: Arc<str>,
    // TODO: generate new client on Deserialize
    // #[serde(skip_serializing)]
    // client: reqwest::Client,
    // TODO: figure out device info
    // #[serde(skip_serializing)]
    //device: Arc<str>,
    // #[serde(skip_serializing)]
    //device_name: Arc<str>,
}

fn default_session() -> Arc<str> {
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
    }

    pub(crate) fn refresh_all_unchecked(&mut self) {
        debug!("refreshing all plex data");
        self.refresh_resources().ok();
        self.refresh_libraries().ok();
    }

    pub(crate) fn refresh_resources(&mut self) -> Result<()> {
        debug!("refreshing resources");
        if self.has_user() {
            let client = self.create_client()?; // TODO: shared client in state
            let resources = PlexResource::list(&client)?;
            self.resources = Some(resources);
            Ok(())
        } else {
            Err(Error::NotAuthenticated)
        }
    }

    pub(crate) fn refresh_libraries(&mut self) -> Result<()> {
        debug!("refreshing libraries");
        if self.has_user() {
            if let Some(server) = &self.selected_server {
                let client = self.create_client()?; // TODO: shared client in state
                self.libraries = Some(Library::list(&client, server.uri.as_ref())?);
                Ok(())
            } else {
                Err(Error::NoServerSelected)
            }
        } else {
            Err(Error::NotAuthenticated)
        }
    }

    pub(crate) fn get_servers(&self) -> Result<Arc<[Arc<str>]>> {
        debug!("listing resources");
        let resources = if let Some(resources) = &self.resources {
            resources
        } else {
            warn!("No resources found when getting servers");
            return Err(Error::NoResourcesFound);
        };

        //let client = self.create_client()?; // TODO: shared client in state
        Ok(resources
            .iter()
            //.filter(|res| res.get_first_working_connection(&client).is_ok()) // TODO fix blocking requests
            .map(|resource| resource.name.clone())
            .collect())
    }

    pub(crate) fn select_server(&mut self, server: &str) -> Result<()> {
        debug!("selecting resource");
        let resources = if let Some(resources) = &self.resources {
            resources
        } else {
            warn!("No resources found when selecting servers");
            return Err(Error::NoResourcesFound);
        };

        let client = self.create_client()?; // TODO: shared client in state
        let server = resources
            .iter()
            .find(|res| res.name == server.into())
            .ok_or(Error::InvalidSeverName)?
            .get_first_working_connection(&client)
            .map(|conn| PlexServer {
                name: server.into(),
                uri: conn.uri.clone(),
            })?;

        self.selected_server = Some(server);

        Ok(())
    }

    pub(crate) fn get_selected_server(&self) -> Option<Arc<str>> {
        self.selected_server
            .as_ref()
            .map(|server| server.name.clone())
    }

    pub(crate) fn reset_server_selection(&mut self) {
        debug!("reseting selected resource");
        self.selected_server = None;
    }

    pub(crate) fn get_libraries(&self) -> Result<Arc<[Arc<str>]>> {
        if let Some(libraries) = &self.libraries {
            debug!("get loaded libraries");

            Ok(libraries.iter().map(|lib| lib.title.clone()).collect()) // maybe filter by music category
        } else {
            Err(Error::NoLibrariesFound)
        }
    }

    pub(crate) fn select_library(&mut self, server: &str) -> Result<()> {
        debug!("selecting library");
        let libraries = if let Some(libraries) = &self.libraries {
            libraries
        } else {
            warn!("No library found when selecting servers");
            return Err(Error::NoLibrariesFound);
        };

        let library = libraries
            .iter()
            .find(|res| res.title == server.into())
            .ok_or(Error::InvalidLibraryName)
            .map(|lib| PlexLibrary {
                key: lib.key.clone(),
                name: lib.title.clone(),
            })?;

        self.selected_library = Some(library);

        Ok(())
    }

    pub(crate) fn get_selected_library(&self) -> Option<Arc<str>> {
        self.selected_library
            .as_ref()
            .map(|library| library.name.clone())
    }

    pub(crate) fn reset_library_selection(&mut self) {
        debug!("reseting selected library");
        self.selected_library = None;
    }
}

#[derive(Deserialize, Clone, Debug)]
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

    pub(crate) fn get_pin(&self) -> Arc<str> {
        self.code.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PlexConnections {
    uri: Arc<str>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PlexResource {
    name: Arc<str>,
    connections: Arc<[PlexConnections]>,
}

impl PlexResource {
    fn list(client: &reqwest::blocking::Client) -> Result<Arc<[Self]>> {
        let url = "https://plex.tv/api/v2/resources";
        Ok(client.get(url).send()?.json()?)
    }

    fn get_first_working_connection(
        &self,
        client: &reqwest::blocking::Client,
    ) -> Result<&PlexConnections> {
        self.connections
            .iter()
            .find(|conn| client.get(conn.uri.as_ref()).send().is_ok())
            .ok_or(Error::NoValidConnections)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Library {
    title: Arc<str>,
    key: Arc<str>,
    #[serde(rename = "type")]
    media_type: Arc<str>, // could be enum
}

impl Library {
    fn list(client: &reqwest::blocking::Client, uri: &str) -> Result<Arc<[Self]>> {
        let url = format!("{uri}/library/sections/");
        Ok(serde_json::from_value(
            client
                .get(url)
                .send()?
                .json::<Value>()?
                .get("MediaContainer")
                .ok_or(Error::MediaContainerNotFound)?
                .get("Directory")
                .ok_or(Error::LibraryDirectoryNotFound)?
                .to_owned(),
        )?)
    }
}
