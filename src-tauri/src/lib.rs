mod handlers;
pub(crate) mod plex;
pub(crate) mod state;

use handlers::*;

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
            settings_state,
            plex_signin,
            plex_check,
            plex_signout,
            plex,
            plex_server,
            plex_update_server
        ])
        .setup(state::setup_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
