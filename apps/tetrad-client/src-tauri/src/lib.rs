mod auth;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let auth_service = auth::AuthService::new();

    tauri::Builder::default()
        .manage(auth::commands::AuthState {
            primary_port: Arc::new(auth_service),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![auth::commands::login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
