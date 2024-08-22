use std::sync::MutexGuard;

use askama::Template;
use log::{debug, info, warn};
use tauri::{AppHandle, Emitter, State};

use crate::{
    plex::Album,
    state::{AppSettings, AppState, Book, Books, InnerAppState, ReadingState},
    Error,
};

use super::Result;

// might move templates to seperate rs file
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[tauri::command]
pub(crate) fn home(_state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `home`");
    let hello = HomeTemplate {};
    Ok(hello.render()?)
}

#[derive(Template)]
#[template(path = "library.html")]
struct LibraryTemplate;

#[tauri::command]
pub(crate) fn library() -> Result<String> {
    debug!("Requesting `library`");
    let library = LibraryTemplate;

    Ok(library.render()?)
}

#[derive(Template)]
#[template(path = "library/pagination.html")]
struct LibraryPaginationTemplate<'a> {
    books: &'a [Album],
    next: usize,
}

#[tauri::command]
pub(crate) fn library_pagination(state: State<'_, AppState>, current: &str) -> Result<String> {
    debug!("Requesting `library_pagination` at {current:?}");
    let current: usize = current.parse()?;
    let state = state.lock()?;
    let page_size = 12; // more likely to fit evenly into the display

    let books = &state.settings.plex.get_albums()?[current..(current + page_size)];

    let next = if books.len() == page_size {
        current + page_size
    } else {
        0
    };

    let library = LibraryPaginationTemplate { books, next };

    Ok(library.render()?)
}

#[derive(Template)]
#[template(path = "library/book.html")]
struct BookTemplate {
    book: Album,
    downloaded: bool,
}

#[tauri::command]
pub(crate) fn book(state: State<'_, AppState>, key: &str) -> Result<String> {
    debug!("Requesting `book` at {key:?}");
    let state = state.lock()?;
    let book = BookTemplate {
        book: state.settings.plex.get_album(key)?,
        downloaded: false, // todo check against downloaded books in state
    };

    Ok(book.render()?)
}

const UPDATE_DOWNLOADED_EVENT: &str = "update-downloaded";
#[tauri::command]
pub(crate) fn plex_download_book(
    state: State<'_, AppState>,
    key: &str,
    app: AppHandle,
) -> Result<()> {
    debug!("Requesting `plex_download_book` at {key:?}");
    let mut state = state.lock()?;

    let album = state.settings.plex.get_album(key)?;
    let new_book = state.books.download_book(album)?;

    if new_book {
        state.save_books();
    } else {
        state.save_book(key);
    }

    app.emit(UPDATE_DOWNLOADED_EVENT, ())?;

    Ok(())
}

#[tauri::command]
pub(crate) fn plex_delete_book(
    state: State<'_, AppState>,
    key: &str,
    app: AppHandle,
) -> Result<()> {
    debug!("Requesting `plex_delete_book` at {key:?}");
    let mut state = state.lock()?;

    state.books.remove_download(key)?;
    state.save_book(key);

    app.emit(UPDATE_DOWNLOADED_EVENT, ())?;

    Ok(())
}

#[derive(Template)]
#[template(path = "player.html")]
struct PlayerTemplate<'a> {
    book: &'a Book,
}

const UPDATE_PLAYER_EVENT: &str = "update-player";

fn create_player(mut state: MutexGuard<InnerAppState>, key: &str) -> Result<String> {
    let album = state.settings.plex.get_album(key)?;
    state.current_book = Some(album.rating_key.clone());
    let (book, new_book) = state.books.get_book_or_insert(album)?;
    book.state = ReadingState::Playing;

    let book = PlayerTemplate {
        book: &book.clone(),
    };

    state.save_current_book();
    if new_book {
        state.save_books();
    }

    Ok(book.render()?)
}

#[tauri::command]
pub(crate) fn start_playing(
    state: State<'_, AppState>,
    app: AppHandle,
    key: &str,
    chapter: Option<&str>,
    _paused: Option<&str>,
) -> Result<String> {
    debug!("Requesting `book` at {key:?}");
    let mut state = state.lock()?;
    let _chapter = chapter.unwrap_or("0");

    if let Some(current) = &state.current_book.clone() {
        debug!("Requesting `book` at {key:?}");
        if current.as_ref() != key {
            if let Some(old) = state.books.get_mut(current) {
                old.state = ReadingState::Paused; // maybe should be something like UnLoaded
            }
            app.emit(UPDATE_PLAYER_EVENT, ())?;
            create_player(state, key)
        } else {
            if let Some(current) = state.books.get_mut(current) {
                current.state = match current.state {
                    ReadingState::Playing => ReadingState::Paused,
                    _ => ReadingState::Playing,
                };
            }
            app.emit("toggle-playing", ())?;

            Err(Error::NoChange) // unsure on if this should be Error or just an empty string...
        }
    } else {
        create_player(state, key)
    }
}

const UPDATE_SETTINGS_EVENT: &str = "update-settings";

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate;

#[tauri::command]
pub(crate) fn settings() -> Result<String> {
    debug!("Requesting `settings`");
    Ok(SettingsTemplate.render()?)
}

#[derive(Template)]
#[template(path = "settings/state.html")]
struct SettingsStateTemplate<'a> {
    settings: &'a AppSettings,
}

#[tauri::command]
pub(crate) fn settings_state(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `settings_state`");
    let state = state.lock()?;
    let state = SettingsStateTemplate {
        settings: &state.settings,
    };
    Ok(state.render()?)
}

#[derive(Template)]
#[template(path = "settings/plex/pin.html")]
struct PinTemplate<'a> {
    pin: &'a str,
}

#[derive(Template)]
#[template(path = "settings/plex/signed_in.html")]
struct PlexSignedInTemplate;

#[derive(Template)]
#[template(path = "settings/plex/signed_out.html")]
struct PlexSignedOutTemplate;

#[tauri::command]
pub(crate) fn plex_signin(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_signin`");
    let mut state = state.lock()?;
    let pin = state.settings.plex.create_login_pin()?;
    let pin_html = PinTemplate { pin: pin.get_pin() };
    let pin_html = pin_html.render()?;
    state.plex_pin = Some(pin); // todo
    Ok(pin_html)
}

#[tauri::command]
pub(crate) fn plex_check(state: State<'_, AppState>, app: AppHandle) -> Result<String> {
    debug!("Requesting `plex_check`");
    let mut state = state.lock()?;
    let plex = if let Some(pin) = state.plex_pin.clone() {
        match state.settings.plex.check_pin(&pin) {
            Ok(_) => {
                info!("Plex signin successful");
                state.save_settings();
                app.emit(UPDATE_SETTINGS_EVENT, ())?; // move this to state/settings struct?
                PlexSignedInTemplate.render()
            }
            Err(crate::plex::Error::WaitingOnPin) => {
                debug!("Waiting for plex pin complete or retry");
                PinTemplate { pin: pin.get_pin() }.render()
            }
            Err(_) => {
                warn!("Plex pin unsuccessful");
                state.plex_pin = None;
                PlexSignedOutTemplate.render()
            }
        }
    } else {
        info!("Plex signin unsuccessful");
        PlexSignedOutTemplate.render()
    }?;

    Ok(plex)
}

#[tauri::command]
pub(crate) fn plex_signout(state: State<'_, AppState>, app: AppHandle) -> Result<String> {
    debug!("Requesting `plex_signout`");
    let mut state = state.lock()?;
    state.settings.plex.signout();
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ())?; // move this to state/settings struct?

    Ok(PlexSignedOutTemplate.render()?)
}

#[tauri::command]
pub(crate) fn plex(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex`");
    let state = state.lock()?;
    let plex = if state.settings.plex.has_user() {
        PlexSignedInTemplate.render()
    } else if let Some(pin) = &state.plex_pin {
        PinTemplate { pin: pin.get_pin() }.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    Ok(plex?)
}

#[derive(Template)]
#[template(path = "settings/plex/server.html")]
struct PlexServerTemplate<'a> {
    urls: Box<[&'a str]>,
    selected: Option<&'a str>,
}

#[tauri::command]
pub(crate) fn plex_server(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_server`");
    let state = state.lock()?;

    let servers = state.settings.plex.get_servers().unwrap_or([].into());
    Ok(PlexServerTemplate {
        urls: servers,
        selected: state.settings.plex.get_selected_server(),
    }
    .render()?)
}

#[tauri::command]
pub(crate) fn plex_update_server(
    state: State<'_, AppState>,
    server: Option<&str>,
    app: AppHandle,
) -> Result<()> {
    debug!("Requesting `plex_update_server`");
    let mut state = state.lock()?;

    if let Some(server) = server {
        state.settings.plex.select_server(server)?; // maybe should error handle on this
    } else {
        state.settings.plex.reset_server_selection();
    }
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ())?; // move this to state/settings struct?

    Ok(())
}

#[derive(Template)]
#[template(path = "settings/plex/library.html")]
struct PlexLibraryTemplate<'a> {
    libraries: Box<[&'a str]>,
    selected: Option<&'a str>,
}

#[tauri::command]
pub(crate) fn plex_library(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_library`");
    let state = state.lock()?;

    let libraries = state.settings.plex.get_libraries().unwrap_or([].into());
    Ok(PlexLibraryTemplate {
        libraries,
        selected: state.settings.plex.get_selected_library(),
    }
    .render()?)
}

#[tauri::command]
pub(crate) fn plex_update_library(
    state: State<'_, AppState>,
    library: Option<&str>,
    app: AppHandle,
) -> Result<()> {
    debug!("Requesting `plex_update_library`");
    let mut state = state.lock()?;

    if let Some(library) = library {
        state.settings.plex.select_library(library)?; // maybe should error handle on this
    } else {
        state.settings.plex.reset_library_selection();
    }
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ())?; // move this to state/settings struct?

    Ok(())
}
