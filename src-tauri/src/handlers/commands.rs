use std::sync::Arc;

use askama::Template;
use log::{debug, info, warn};
use tauri::{AppHandle, Emitter, State};

use crate::{
    plex::Album,
    state::{AppSettings, AppState},
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
struct LibraryTemplate {
    books: Arc<[Album]>,
}

#[tauri::command]
pub(crate) fn library(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `library`");
    let state = state.lock()?;
    let library = LibraryTemplate {
        books: state.settings.plex.get_albums()?,
    };

    Ok(library.render()?)
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
struct PinTemplate {
    pin: Arc<str>,
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
                app.emit(UPDATE_SETTINGS_EVENT, ()).ok(); // move this to state/settings struct?
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
    };

    Ok(plex?)
}

#[tauri::command]
pub(crate) fn plex_signout(state: State<'_, AppState>, app: AppHandle) -> Result<String> {
    debug!("Requesting `plex_signout`");
    let mut state = state.lock()?;
    state.settings.plex.signout();
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ()).ok(); // move this to state/settings struct?

    Ok(PlexSignedOutTemplate.render()?)
}

#[tauri::command]
pub(crate) fn plex(state: State<'_, AppState>) -> Result<String> {
    debug!("Requesting `plex`");
    let state = state.lock()?;
    let plex = if state.settings.plex.has_user() {
        PlexSignedInTemplate.render()
    } else if let Some(pin) = state.plex_pin.clone() {
        PinTemplate { pin: pin.get_pin() }.render()
    } else {
        PlexSignedOutTemplate.render()
    };
    Ok(plex?)
}

#[derive(Template)]
#[template(path = "settings/plex/server.html")]
struct PlexServerTemplate {
    urls: Arc<[Arc<str>]>,
    selected: Option<Arc<str>>,
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
        state.settings.plex.select_server(server).ok(); // maybe should error handle on this
    } else {
        state.settings.plex.reset_server_selection();
    }
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ()).ok(); // move this to state/settings struct?

    Ok(())
}

#[derive(Template)]
#[template(path = "settings/plex/library.html")]
struct PlexLibraryTemplate {
    libraries: Arc<[Arc<str>]>,
    selected: Option<Arc<str>>,
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
        state.settings.plex.select_library(library).ok(); // maybe should error handle on this
    } else {
        state.settings.plex.reset_library_selection();
    }
    state.save_settings();
    app.emit(UPDATE_SETTINGS_EVENT, ()).ok(); // move this to state/settings struct?

    Ok(())
}
