use askama::Template;
use log::{debug, info, warn};
use tauri::State;

use crate::{
    plex,
    state::{AppSettings, AppState, Books},
};

use super::Result;

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
pub(crate) fn home(_state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `home`");
    let hello = HomeTemplate {};
    Ok(hello.render()?)
}

#[tauri::command]
pub(crate) fn library(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `library`");
    let state = state.lock()?;
    let library = LibraryTemplate {
        books: &state.books,
    };

    Ok(library.render()?)
}

#[tauri::command]
pub(crate) fn settings(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `settings`");
    let state = state.lock()?;
    let settings = SettingsTemplate {
        settings: &state.settings,
    };
    Ok(settings.render()?)
}

#[tauri::command]
pub(crate) fn plex_signin(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_signin`");
    let mut state = state.lock()?;
    let pin = state.settings.plex.create_login_pin()?;
    let pin_html = PinTemplate { pin: pin.ref_pin() };
    let pin_html = pin_html.render()?;
    state.plex_pin = Some(pin); // todo
    Ok(pin_html)
}

#[tauri::command]
pub(crate) fn plex_check(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_check`");
    let mut state = state.lock()?;
    let plex = if let Some(pin) = state.plex_pin.clone() {
        match state.settings.plex.check_pin(&pin) {
            Ok(_) => {
                info!("Plex signin successful");
                state.save_settings();
                PlexSignedInTemplate.render()
            }
            Err(plex::Error::WaitingOnPin) => {
                debug!("Waiting for plex pin complete or retry");
                PinTemplate { pin: pin.ref_pin() }.render()
            }
            Err(_) => {
                warn!("Plex pin unsuccessful");
                PlexSignedOutTemplate.render()
            }
        }
    } else {
        info!("Plex signin unsuccessful");
        PlexSignedOutTemplate.render()
    };

    Ok(plex?)
}

#[tauri::command]
pub(crate) fn plex_signout(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex_signout`");
    let mut state = state.lock()?;
    state.settings.plex.signout();
    state.save_settings();

    Ok(PlexSignedOutTemplate.render()?)
}

#[tauri::command]
pub(crate) fn plex(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex`");
    let state = state.lock()?;
    let plex = if state.settings.plex.has_user() {
        PlexSignedInTemplate.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    Ok(plex?)
}
