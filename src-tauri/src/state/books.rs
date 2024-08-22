use std::{collections::HashMap, sync::Arc};

use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::plex::Album;

use super::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
enum ReadingState {
    Playing,
    Paused,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Book {
    book_details: Album,
    state: ReadingState,
    progress: f64,
    downloaded: Option<Box<str>>,
}

impl Book {
    // pub(crate) fn new(album: Album) -> Self {
    //     Book {
    //         book_details: album,
    //         state: ReadingState::Playing,
    //         progress: 0f64,
    //         downloaded: None,
    //     }
    // }

    const CURRENT_BOOK_STORE: &'static str = "current-book";
    pub(super) fn get_current(store: &Store<Wry>) -> Option<Arc<str>> {
        debug!("Loading {} store", Self::CURRENT_BOOK_STORE);
        let book = if let Some(book) = store.get(Self::CURRENT_BOOK_STORE) {
            serde_json::from_value(book.to_owned()).map_err(|err| err.into())
        } else {
            Err(Error::StoreEmpty)
        };

        book.ok()
    }

    pub(super) fn from_key(store: &Store<Wry>, key: &str) -> Result<Self> {
        let store_key = format!("book:{key}");
        debug!("Loading {store_key} store");
        if let Some(book) = store.get(store_key) {
            serde_json::from_value(book.to_owned()).map_err(|err| err.into())
        } else {
            Err(Error::StoreEmpty)
        }
    }

    // pub(super) fn save(&self, store: &mut Store<Wry>) -> Result<()> {
    //     self.__save(store)?;
    //     store.save()?;

    //     Ok(())
    // }

    fn __save(&self, store: &mut Store<Wry>) -> Result<()> {
        let key = self.book_details.rating_key.as_ref();
        let store_key = format!("book:{key}");
        debug!("saving {store_key} store");
        store.insert(store_key, serde_json::to_value(self).unwrap_or_default())?;
        store.save()?;

        Ok(())
    }

    const ALL_BOOKS_STORE: &'static str = "all-books";
    pub(super) fn get_all_books(store: &mut Store<Wry>) -> HashMap<Arc<str>, Self> {
        debug!("Loading all books");
        let books = if let Some(books) = store.get(Self::ALL_BOOKS_STORE) {
            serde_json::from_value::<Box<[Arc<str>]>>(books.to_owned()).map_err(|err| err.into())
        } else {
            Err(Error::StoreEmpty)
        };

        let books = if let Ok(books) = books {
            books
        } else {
            Box::new([])
        };

        let books = books
            .par_iter()
            .filter_map(|book_key| {
                if let Ok(book) = Self::from_key(store, book_key.as_ref()) {
                    Some((book_key.clone(), book))
                } else {
                    None // if for whatever reason we can't find the key we just assume its gone and remove it from the list
                }
            })
            .collect::<HashMap<Arc<str>, Self>>();
        books.save(store).ok();
        books
    }
}

pub(super) trait Save {
    fn save(&self, store: &mut Store<Wry>) -> Result<()>;
}

impl Save for HashMap<Arc<str>, Book> {
    fn save(&self, store: &mut Store<Wry>) -> Result<()> {
        for book in self.values() {
            book.__save(store).ok();
        }

        let books = self.keys().cloned().collect::<Box<[Arc<str>]>>();
        store.insert(
            Book::ALL_BOOKS_STORE.to_string(),
            serde_json::to_value(books).unwrap_or_default(),
        )?;
        store.save()?;

        Ok(())
    }
}
