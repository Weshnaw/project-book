use std::{collections::HashMap, sync::Arc};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

#[derive(Serialize, Deserialize, Display, Debug)]
#[display(fmt = "{:#?}", "self")]
pub(crate) struct Books {
    pub(crate) books: HashMap<Arc<str>, Book>,
}

impl Books {
    pub(super) fn from_store(store: &mut Store<Wry>) -> Self {
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

#[derive(Serialize, Deserialize, Display, Debug)]
pub(crate) struct Book;

impl Book {
    pub(super) fn from_store(store: &mut Store<Wry>, id: &str) -> Self {
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
struct BookIds(Vec<Arc<str>>);

impl BookIds {
    const STORE: &'static str = "books";
    pub(super) fn from_store(store: &mut Store<Wry>) -> Self {
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
