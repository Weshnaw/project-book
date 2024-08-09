#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use askama::Template;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State, Wry};
use tauri_plugin_store::{Store, StoreBuilder};

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

#[derive(Serialize, Deserialize, Display)]
struct AppSettings;

impl AppSettings {
    const STORE: &'static str = "settings";
    fn from_store(store: &mut Store<Wry>) -> Self {
        if let Some(settings) = store.get(Self::STORE) {
            serde_json::from_value(settings.to_owned()).unwrap() // TODO: handle gracefully
        } else {
            let settings = Self;
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
}
type AppState = Mutex<InnerAppState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![home, library, settings])
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
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
