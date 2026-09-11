use std::sync::Arc;

use tauri::State;
use tetrad_auth::ports::primary::AuthUseCases;

pub struct AuthState {
    pub primary_port: Arc<dyn AuthUseCases>
}

#[tauri::command]
pub async fn login(
    auth_state: State<'_, AuthState>
) -> Result<i64, String> {
    auth_state.primary_port.login().await;

    Ok(1)
}
