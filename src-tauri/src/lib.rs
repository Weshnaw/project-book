use std::path::PathBuf;

use serde_json::json;
use tauri::{Manager, Wry};
use tauri_plugin_store::{with_store, StoreCollection};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let stores = app.app_handle().state::<StoreCollection<Wry>>();
            let path = PathBuf::from("store.bin");

            with_store(app.app_handle().clone(), stores, path, |store| {
                // Note that values must be serde_json::Value instances,
                // otherwise, they will not be compatible with the JavaScript bindings.
                store.insert("some-key".to_string(), json!({ "value": 5 }))?;

                // Get a value from the store.
                let value = store
                    .get("some-key")
                    .expect("Failed to get value from store");
                println!("{}", value); // {"value":5}

                // You can manually save the store after making changes.
                // Otherwise, it will save upon graceful exit as described above.
                store.save()?;

                Ok(())
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
