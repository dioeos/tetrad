use axum_test::TestResponse;
use serde_json::Value;
use tetrad_server::Config;

pub fn test_config(db_url: String) -> Config {
    Config::new(
        db_url,
        "127.0.0.1:0".parse().unwrap(),
        "integration-test-instance",
        "http://localhost",
    )
}

pub fn test_database_url() -> String {
    let path = std::env::temp_dir().join(format!(
        "tetrad-server-test-{}.sqlite3",
        uuid::Uuid::now_v7()
    ));

    std::fs::File::create(&path).unwrap();

    format!("sqlite://{}", path.display())
}

pub fn response_body(response: &TestResponse) -> Value {
    response.json::<Value>()
}
