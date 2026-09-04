// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tetrad_api_contract::ApiErrorResponse;

const API_BASE_URL: &str = "http://localhost:8080";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn register(email: String, password: String) -> Result<(), ApiErrorResponse> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, register])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
