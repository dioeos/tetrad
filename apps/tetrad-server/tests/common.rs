use axum_test::TestResponse;
use serde_json::Value;
use tetrad_server::Config;

pub fn test_config() -> Config {
    Config::new(
        "sqlite::memory:",
        "127.0.0.1:0".parse().unwrap(),
        "integration-test-instance",
        "http://localhost",
    )
}

pub fn response_body(response: &TestResponse) -> Value {
    response.json::<Value>()
}
