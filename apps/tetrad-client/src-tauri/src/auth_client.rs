use tetrad_api_contract::ApiErrorResponse;

use super::transport::ReqwestJsonClient;

const TORII_AUTH_BASE_URL: &str = "http://localhost:8080/auth";

//@NOTE: An HTTP client that makes use of the `ReqwestJsonClient` transport layer
//       to interact with Torii's defined auth routes. In the `torii-client` crate,
//       there are types that are useful when using Torii; however, in order to prevent
//       tight coupling with Torii's endpoint definition, there is a separate client
//       that interacts and expects specifically Torii `Endpoint` structures
//
//       References:
//       https://github.com/cmackenzie1/torii-rs/blob/main/torii-client/src/lib.rs

pub struct ToriiAuthClient {
    http: ReqwestJsonClient,
    base_url: String,
}

impl ToriiAuthClient {
    pub fn new() -> Self {
        Self {
            http: ReqwestJsonClient::new(),
            base_url: TORII_AUTH_BASE_URL.to_owned(),
        }
    }

    pub async fn call<R>(
        &self,
        request: &R,
    ) -> Result<<R::Endpoint as torii_client::Endpoint>::Response, ApiErrorResponse>
    where
        R: torii_client::EndpointRequest,
    {
        self.call_endpoint::<R::Endpoint>(request).await
    }

    pub async fn call_endpoint<E>(
        &self,
        request: &E::Request,
    ) -> Result<E::Response, ApiErrorResponse>
    where
        E: torii_client::Endpoint,
    {
        let url = format!("{}{}", self.base_url, E::SPEC.path);

        self.http
            .send_json(torii_method_to_reqwest(E::SPEC.method), url, request)
            .await
    }
}

fn torii_method_to_reqwest(method: torii_client::Method) -> reqwest::Method {
    match method {
        torii_client::Method::Get => reqwest::Method::GET,
        torii_client::Method::Post => reqwest::Method::POST,
        torii_client::Method::Delete => reqwest::Method::DELETE,
    }
}
