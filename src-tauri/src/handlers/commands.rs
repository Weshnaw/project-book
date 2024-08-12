use askama::Template;
use tauri::State;

use crate::state::{AppSettings, AppState, Books};

// might move templates to seperate rs file
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate<'a> {
    settings: &'a AppSettings,
}

#[derive(Template)]
#[template(path = "library.html")]
struct LibraryTemplate<'a> {
    books: &'a Books,
}

#[derive(Template)]
#[template(path = "settings/pin.html")]
struct PinTemplate<'a> {
    pin: &'a str,
}

#[derive(Template)]
#[template(path = "settings/plexSignedIn.html")]
struct PlexSignedInTemplate;

#[derive(Template)]
#[template(path = "settings/plexSignedOut.html")]
struct PlexSignedOutTemplate;

#[tauri::command]
pub(crate) fn home(_state: State<'_, AppState>) -> String {
    let hello = HomeTemplate {};
    hello.render().unwrap()
}

#[tauri::command]
pub(crate) fn library(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let library = LibraryTemplate {
        books: &state.books,
    };
    library.render().unwrap()
}

#[tauri::command]
pub(crate) fn settings(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let settings = SettingsTemplate {
        settings: &state.settings,
    };
    settings.render().unwrap()
}

#[tauri::command]
pub(crate) fn plex_signin(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    let pin = state.settings.plex.create_login_pin();
    let pin_html = PinTemplate { pin: pin.ref_pin() };
    let pin_html = pin_html.render().unwrap();
    state.plex_pin = Some(pin);
    pin_html
}

#[tauri::command]
pub(crate) fn plex_check(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    let plex = if let Some(pin) = state.plex_pin.clone() {
        let success = state.settings.plex.check_pin(&pin);
        if success {
            state.save_settings();
            PlexSignedInTemplate.render()
        } else {
            PinTemplate { pin: pin.ref_pin() }.render()
        }
    } else {
        PlexSignedOutTemplate.render()
    };

    plex.unwrap()
}

#[tauri::command]
pub(crate) fn plex_signout(state: State<'_, AppState>) -> String {
    let mut state = state.lock().unwrap();
    state.settings.plex.signout();
    state.save_settings();

    PlexSignedOutTemplate.render().unwrap()
}

#[tauri::command]
pub(crate) fn plex(state: State<'_, AppState>) -> String {
    let state = state.lock().unwrap();
    let plex = if state.settings.plex.has_user() {
        PlexSignedInTemplate.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    plex.unwrap()
}
