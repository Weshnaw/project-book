#![allow(dead_code)]

use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use askama::Template;
use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Wry};
use tauri_plugin_store::{Store, StoreBuilder};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

#[tauri::command]
fn home(_state: State<'_, AppState>) -> String {
    let hello = HomeTemplate {};
    hello.render().unwrap()
}

#[derive(Template)]
#[template(path = "library.html")]
struct LibraryTemplate<'a> {
    books: &'a Books,
}

#[tauri::command]
fn library(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let library = LibraryTemplate {
        books: &state.books,
    };
    library.render().unwrap()
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate<'a> {
    settings: &'a AppSettings,
}

#[tauri::command]
fn settings(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let settings = SettingsTemplate {
        settings: &state.settings,
    };
    settings.render().unwrap()
}

#[derive(Serialize, Deserialize, Display, Default)]
struct AppSettings {
    plex: Plex,
}

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
struct Plex {
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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PlexPin {
    id: u64,
    code: Arc<str>,
    auth_token: Option<Arc<str>>,
}

impl Plex {
    fn create_client(&self) -> reqwest::blocking::Client {
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
            HeaderValue::from_str(self.client_ident.as_ref()).unwrap(),
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
            HeaderValue::from_str(self.session_token.as_ref()).unwrap(),
        );
        // headers.insert("X-Plex-Device",
        //     HeaderValue::from_str(self.device.as_ref()).unwrap(),

        // );
        // headers.insert("X-Plex-Device-Name",
        //     HeaderValue::from_str(self.device_name.as_ref()).unwrap(),
        // );
        if let Some(token) = &self.user_token {
            headers.insert(
                "X-Plex-Token",
                HeaderValue::from_str(token.as_ref()).unwrap(),
            );
        }

        reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    }

    fn create_login_pin(&self) -> PlexPin {
        let client = self.create_client(); // TODO: shared client in state

        PlexPin::new(&client)
    }

    fn check_pin(&mut self, pin: &PlexPin) -> bool {
        let client = self.create_client(); // TODO: shared client in state
        let checked_pin = pin.check_pin(&client);
        self.user_token = checked_pin.auth_token;
        self.user_token.is_some()
    }
}

impl PlexPin {
    fn new(client: &reqwest::blocking::Client) -> Self {
        let pin_url = "https://plex.tv/api/v2/pins";
        client
            .post(pin_url)
            .send()
            .unwrap()
            .json::<PlexPin>()
            .unwrap() // TODO: better error handling
    }

    fn check_pin(&self, client: &reqwest::blocking::Client) -> Self {
        let pin_url = format!("https://plex.tv/api/v2/pins/{}", self.id);

        client.post(pin_url).send().unwrap().json().unwrap() // TODO: better error handling, handle expired pin
    }
}

#[derive(Template)]
#[template(path = "pin.html")]
struct PinTemplate<'a> {
    pin: &'a str,
}

#[tauri::command]
fn plex_signin(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    //let pin = state.settings.plex.create_login_pin();
    let pin = PlexPin {
        id: 1,
        code: "ABCD".into(),
        auth_token: None,
    };
    let pin_html = PinTemplate {
        pin: pin.code.as_ref(),
    };
    let pin_html = pin_html.render().unwrap();
    state.plex_pin = Some(pin);
    pin_html
}

#[tauri::command]
fn plex_check(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    let plex = if let Some(plex_pin) = state.plex_pin.clone() {
        //let success = state.settings.plex.check_pin(&plex_pin);
        state.settings.plex.user_token = Some("A".into());
        let success = false;
        if success {
            state.save_settings();
            PlexSignedInTemplate.render()
        } else {
            PinTemplate {
                pin: plex_pin.code.as_ref(),
            }
            .render()
        }
    } else {
        PlexSignedOutTemplate.render()
    };

    plex.unwrap()
}

#[tauri::command]
fn plex_signout(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    state.settings.plex.user_token = None;
    state.save_settings();

    PlexSignedOutTemplate.render().unwrap()
}

#[derive(Template)]
#[template(path = "plexSignedIn.html")]
struct PlexSignedInTemplate;
#[derive(Template)]
#[template(path = "plexSignedOut.html")]
struct PlexSignedOutTemplate;
#[tauri::command]
fn plex(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let plex = if state.settings.plex.user_token.is_some() {
        PlexSignedInTemplate.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    plex.unwrap()
}

impl AppSettings {
    const STORE: &'static str = "settings";
    fn from_store(store: &mut Store<Wry>) -> Self {
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

    fn save(&self, store: &mut Store<Wry>) {
        store
            .insert(
                Self::STORE.to_string(),
                serde_json::to_value(self).unwrap_or_default(),
            )
            .ok();
        store.save().ok();
    }
}

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
struct BookIds(Vec<Arc<str>>);

impl BookIds {
    const STORE: &'static str = "books";
    fn from_store(store: &mut Store<Wry>) -> Self {
        if let Some(books) = store.get(Self::STORE) {
            serde_json::from_value(books.to_owned()).unwrap() // TODO: handle gracefully
        } else {
            let books = Self(vec!["a".into(), "b".into(), "c".into()]); // Debug books
            store
                .insert(
                    Self::STORE.to_string(),
                    serde_json::to_value(&books).unwrap_or_default(),
                )
                .ok();
            store.save().ok();

            books
        }
    }
}

#[derive(Serialize, Deserialize, Display, Debug)]
struct Book;
impl Book {
    fn from_store(store: &mut Store<Wry>, id: &str) -> Self {
        if let Some(books) = store.get(id) {
            serde_json::from_value(books.to_owned()).unwrap() // TODO: handle gracefully
        } else {
            let books = Self;
            store
                .insert(
                    id.to_string(),
                    serde_json::to_value(&books).unwrap_or_default(),
                )
                .ok();
            store.save().ok();

            books
        }
    }
}

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
struct Books {
    books: HashMap<Arc<str>, Book>,
}

impl Books {
    fn from_store(store: &mut Store<Wry>) -> Self {
        let book_ids = BookIds::from_store(store);

        let books = book_ids
            .0
            .into_iter()
            .map(|id| {
                let book = Book::from_store(store, &id);
                (id, book)
            })
            .collect();

        Self { books }
    }
}

struct InnerAppState {
    settings: AppSettings,
    books: Books,
    store: Store<Wry>,
    plex_pin: Option<PlexPin>,
}

impl InnerAppState {
    fn save_settings(&mut self) {
        self.settings.save(&mut self.store)
    }
}
type AppState = Mutex<InnerAppState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            home,
            library,
            settings,
            plex_signin,
            plex_check,
            plex_signout,
            plex
        ])
        .setup(|app| {
            let mut store = StoreBuilder::new("store.bin").build(app.handle().clone());
            if true {
                // reset store
                store.reset().ok();
                store.clear().ok();
                store.save().ok();
            }
            store.load().ok();
            let settings = AppSettings::from_store(&mut store);
            let books = Books::from_store(&mut store);

            app.manage(Mutex::new(InnerAppState {
                settings,
                books,
                store,
                plex_pin: None,
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
