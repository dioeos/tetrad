// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod auth_client;
mod state;
mod transport;

use auth_client::ToriiAuthClient;
use state::ClientState;

use tauri::State;
use tetrad_api_contract::ApiErrorResponse;

#[tauri::command]
async fn register(
    state: State<'_, ClientState>,
    email: String,
    password: String,
) -> Result<torii_client::MessageResponse, ApiErrorResponse> {
    let request = torii_client::RegisterRequest { email, password };
    state.torii_client.call(&request).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ClientState {
            torii_client: ToriiAuthClient::new(),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![register])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
