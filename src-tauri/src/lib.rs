mod handlers;
pub(crate) mod plex;
pub(crate) mod state;

use handlers::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("project_book_htmx_lib", log::LevelFilter::Trace)
                .level_for("log@http*", log::LevelFilter::Trace)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            home,
            library,
            library_pagination,
            book,
            plex_download_book,
            plex_delete_book,
            start_playing,
            settings,
            settings_state,
            plex_signin,
            plex_check,
            plex_signout,
            plex,
            plex_server,
            plex_update_server,
            plex_library,
            plex_update_library
        ])
        .setup(state::setup_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
