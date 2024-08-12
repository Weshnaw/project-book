mod error;

pub use error::*;
use log::debug;

use std::{env, sync::Arc};

use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
pub(crate) struct Plex {
    client_ident: Arc<str>,
    user_token: Option<Arc<str>>,
    #[serde(skip_serializing)]
    #[serde(default = "default_session")]
    session_token: Arc<str>,
    // TODO: generate new client on Deserialize
    // #[serde(skip_serializing)]
    // client: reqwest::Client,
    // TODO: figure out device info
    //device: Arc<str>,
    //device_name: Arc<str>,
}

fn default_session() -> Arc<str> {
    Uuid::new_v4().to_string().into()
}

impl Default for Plex {
    fn default() -> Self {
        Self {
            client_ident: Uuid::new_v4().to_string().into(), // I could probably just pass along the uuid itself
            user_token: None,
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
            Ok(())
        } else {
            Err(Error::WaitingOnPin)
        }
    }

    pub(crate) fn has_user(&self) -> bool {
        self.user_token.is_some()
    }

    pub(crate) fn signout(&mut self) {
        self.user_token = None
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
        Ok(client.post(pin_url).send()?.json::<PlexPin>()?)
    }

    fn check_pin(&self, client: &reqwest::blocking::Client) -> Result<Self> {
        let pin_url = format!("https://plex.tv/api/v2/pins/{}", self.id);

        debug!("Checking pin {:#?}", self);
        let res = client.get(pin_url).send()?;

        Ok(res.json()?)
    }

    pub(crate) fn ref_pin(&self) -> &str {
        self.code.as_ref()
    }
}
