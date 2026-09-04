use serde::{Serialize, de::DeserializeOwned};
use tetrad_api_contract::ApiErrorResponse;

//@NOTE: The `Transport` trait define din `torii-client` crate is not used due to the
//       original implementation possessing `?Send`. However, Tauri executes its async
//       commands on its async runtime. If transport has `?Send`, then a command awaiting
//       may fail with an error resembling:
//
//       "future cannot be sent between threads safely"
//
//       Tauri's runtime also explictly requires spawned futures to implement Send,
//       as async commands are executed on separate async task using `async_runtime::spawn`
//
//
//       References:
//       https://v2.tauri.app/develop/calling-rust
//       https://github.com/cmackenzie1/torii-rs/blob/main/torii-client/src/lib.rs

#[derive(Clone)]
pub struct ReqwestJsonClient {
    client: reqwest::Client,
}

impl ReqwestJsonClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_json<Request, Response>(
        &self,
        method: reqwest::Method,
        url: String,
        request: &Request,
    ) -> Result<Response, ApiErrorResponse>
    where
        Request: Serialize + ?Sized,
        Response: DeserializeOwned,
    {
        let response = self
            .client
            .request(method, url)
            .json(request)
            .send()
            .await
            .map_err(|error| ApiErrorResponse {
                code: "network_error".to_owned(),
                message: error.to_string(),
            })?;

        let status = response.status();

        if !status.is_success() {
            let fallback = ApiErrorResponse {
                code: status.as_u16().to_string(),
                message: status
                    .canonical_reason()
                    .unwrap_or("request failed")
                    .to_owned(),
            };
            return Err(response
                .json::<ApiErrorResponse>()
                .await
                .unwrap_or(fallback));
        }

        response
            .json::<Response>()
            .await
            .map_err(|error| ApiErrorResponse {
                code: "invalid_response".to_owned(),
                message: error.to_string(),
            })
    }
}
