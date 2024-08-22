use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use log::debug;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::plex::Album;

use super::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum ReadingState {
    Playing,
    Paused,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Book {
    pub(crate) book_details: Album, // honestly this should probably be a key and the album list should be a hashmap
    pub(crate) state: ReadingState,
    pub(crate) progress: f64,
    downloaded: Option<Arc<str>>,
}

impl Book {
    pub(crate) fn new(album: Album) -> Self {
        Book {
            book_details: album,
            state: ReadingState::Paused,
            progress: 0f64,
            downloaded: None,
        }
    }

    pub(super) const CURRENT_BOOK_STORE: &'static str = "current-book";
    pub(super) fn _get_current(store: &Store<Wry>) -> Option<Arc<str>> {
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

    pub(crate) fn save(&self, store: &mut Store<Wry>) -> Result<()> {
        self.__save(store)?;
        store.save()?;

        Ok(())
    }

    fn __save(&self, store: &mut Store<Wry>) -> Result<()> {
        let key = self.book_details.rating_key.as_ref();
        let store_key = format!("book:{key}");
        debug!("saving {store_key} store");
        store.insert(store_key, serde_json::to_value(self).unwrap_or_default())?;

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

    pub(crate) fn download(&mut self) -> Result<()> {
        todo!("download");
        // let files = self.book_details.get_file_locations();
        // if files.len() >= 1 {
        //   let base_folder = self.book_details.rating_key.as_ref();
        //   for file in files {
        //     // download each file to the base_folder
        //   }
        //   self.downloaded = Some(base_folder);
        // } else files.len() == 1 {
        //   let location = self.book_details.rating_key.as_ref();
        //   let file = files.first().unwrap();
        //   // download the file to the location;
        //   self.downloaded = Some(location);
        // }
        // self.save(store)
    }

    pub(crate) fn remove_download(&mut self) -> Result<()> {
        let _location = self.downloaded.as_ref().ok_or(Error::BookNotDownloaded)?;

        todo!("remove from location");

        // self.downloaded = None;

        // self.save(store)
    }
}

pub(crate) trait Books {
    fn save(&self, store: &mut Store<Wry>) -> Result<()>;
    fn get_book_or_insert(&mut self, album: Album) -> Result<(&mut Book, bool)>;
    fn download_book(&mut self, album: Album) -> Result<bool>;
    fn remove_download(&mut self, key: &str) -> Result<()>;
}

impl Books for HashMap<Arc<str>, Book> {
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

    fn get_book_or_insert(&mut self, album: Album) -> Result<(&mut Book, bool)> {
        let mut new_key = false;
        let key = album.rating_key.clone();
        let book = match self.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let book = Book::new(album);

                new_key = true;
                v.insert(book)
            }
        };

        Ok((book, new_key))
    }

    fn download_book(&mut self, album: Album) -> Result<bool> {
        let (book, new_key) = self.get_book_or_insert(album)?;

        book.download()?;
        Ok(new_key)
    }

    fn remove_download(&mut self, key: &str) -> Result<()> {
        let book = self.get_mut(key).ok_or(Error::NoBookFound)?;

        book.remove_download()
    }
}
