use axum_test::TestServer;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tetrad_server::build_app;

mod common;

use common::{response_body, test_config};

const GET_INSTANCE_URL: &str = "/instance";

#[sqlx::test(migrations = "./migrations")]
async fn get_instance_http_endpoint_returns_initialized_instance(pool: SqlitePool) {
    let app = build_app(pool.clone(), test_config()).await.unwrap();

    let server = TestServer::new(app);

    let response = server.get(GET_INSTANCE_URL).await;

    response.assert_status_ok();
    let body = response_body(&response);

    assert_eq!(body["name"], json!("integration-test-instance"));
    assert_eq!(body["setup_completed_at_ms"], Value::Null);

    let id = body["id"]
        .as_str()
        .expect("response should contain a string ID");

    assert!(!id.is_empty());

    let row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(row_count, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_instance_http_endpoint_returns_404_when_no_instance(pool: SqlitePool) {
    let app = build_app(pool.clone(), test_config()).await.unwrap();

    let server = TestServer::new(app);

    sqlx::query("DELETE FROM instances")
        .execute(&pool)
        .await
        .unwrap();

    let response = server.get(GET_INSTANCE_URL).await;

    response.assert_status_not_found();
    let body = response_body(&response);

    assert_eq!(body["code"], json!("not_found"));
    assert_eq!(body["message"], json!("instance not found"))
}

#[sqlx::test(migrations = "./migrations")]
async fn get_instance_http_endpoint_returns_500_on_database_failure(pool: SqlitePool) {
    let app = build_app(pool.clone(), test_config()).await.unwrap();

    let server = TestServer::new(app);

    pool.close().await;

    let response = server.get(GET_INSTANCE_URL).await;
    let body = response_body(&response);

    assert_eq!(body["code"], json!("internal_server_error"));
    assert_eq!(body["message"], json!("failed to load instance"));
}
