use askama::Template;
use log::{debug, info, warn};
use tauri::State;

use crate::state::{AppSettings, AppState, Books};

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
struct LibraryTemplate<'a> {
    books: &'a Books,
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
            Err(crate::plex::Error::WaitingOnPin) => {
                debug!("Waiting for plex pin complete or retry");
                PinTemplate { pin: pin.ref_pin() }.render()
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
    } else if let Some(pin) = state.plex_pin.clone() {
        PinTemplate { pin: pin.ref_pin() }.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    Ok(plex?)
}
