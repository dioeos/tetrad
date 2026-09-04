// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tetrad_api_types::{ApiErrorResponse, RegisterRequest, RegisterDto};

const API_BASE_URL: &str = "http://localhost:8080";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn register(email: String, password: String) -> Result<RegisterDto, ApiErrorResponse> {
    let request = RegisterRequest { email, password };

    let response = reqwest::Client::new()
        .post(format!("{API_BASE_URL}/auth/register"))
        .json(&request)
        .send()
        .await
        .map_err(|_| ApiErrorResponse {
            code: "client_error".to_owned(),
            message: "could not reach server".to_owned(),
        })?;

    let status = response.status();

    if !status.is_success() {
        return Err(response
            .json::<ApiErrorResponse>()
            .await
            .map_err(|_| ApiErrorResponse {
                code: "client_error".to_owned(),
                message: "server returned an unreadable error response".to_owned()
            })?);
    }

    response
        .json::<RegisterDto>()
        .await
        .map_err(|_| ApiErrorResponse {
            code: "client_error".to_owned(),
            message: "server returned an unreadable success response".to_owned()
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, register])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
