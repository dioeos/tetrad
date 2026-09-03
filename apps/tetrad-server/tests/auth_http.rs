use axum::http::{StatusCode, header};
use axum_test::TestServer;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tetrad_server::build_app;
use uuid::Uuid;

mod common;

use common::{response_body, test_config};

const CREATE_USER_URL: &str = "/users";
const LOGIN_URL: &str = "/login";
const ME_URL: &str = "/me";
const PASSWORD: &str = "a-valid-password";
const SESSION_COOKIE_NAME: &str = "id";

async fn test_server(pool: &SqlitePool) -> TestServer {
    let app = build_app(pool.clone(), test_config()).await.unwrap();
    TestServer::builder().save_cookies().build(app)
}

async fn insert_user_with_hash(
    pool: &SqlitePool,
    username: &str,
    normalized_username: &str,
    password_hash: &str,
) -> String {
    let external_id = Uuid::now_v7().to_string();
    let result = sqlx::query(
        r#"
        INSERT INTO users (
            external_id,
            username,
            normalized_username,
            created_at_ms,
            updated_at_ms
        )
        VALUES (?, ?, ?, 1, 1)
        "#,
    )
    .bind(&external_id)
    .bind(username)
    .bind(normalized_username)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO password_credentials (user_id, password_hash, updated_at_ms)
        VALUES (?, ?, 1)
        "#,
    )
    .bind(result.last_insert_rowid())
    .bind(password_hash)
    .execute(pool)
    .await
    .unwrap();

    external_id
}

async fn insert_user(
    pool: &SqlitePool,
    username: &str,
    normalized_username: &str,
    password: &str,
) -> String {
    let password_hash = password_auth::generate_hash(password);
    insert_user_with_hash(pool, username, normalized_username, &password_hash).await
}

fn assert_auth_error(response: &axum_test::TestResponse, code: &str, message: &str) {
    let body = response_body(response);
    assert_eq!(body["code"], json!(code));
    assert_eq!(body["message"], json!(message));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_returns_public_user_and_persists_credentials(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let response = server
        .post(CREATE_USER_URL)
        .json(&json!({
            "username": "Alice_01",
            "password": PASSWORD,
        }))
        .await;

    response.assert_status_ok();
    let body = response_body(&response);
    assert_eq!(body["username"], json!("Alice_01"));
    assert_eq!(
        body.as_object().unwrap().len(),
        2,
        "only public fields should be returned"
    );

    let external_id = body["external_id"]
        .as_str()
        .expect("external_id should be a string");
    Uuid::parse_str(external_id).expect("external_id should be a UUID");

    let row: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT u.external_id, u.username, u.normalized_username, pc.password_hash
        FROM users AS u
        INNER JOIN password_credentials AS pc ON pc.user_id = u.id
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(row.0, external_id);
    assert_eq!(row.1, "Alice_01");
    assert_eq!(row.2, "alice_01");
    assert_ne!(row.3, PASSWORD);
    password_auth::verify_password(PASSWORD, &row.3)
        .expect("the persisted hash should verify the submitted password");
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_trims_username_and_password(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let response = server
        .post(CREATE_USER_URL)
        .json(&json!({
            "username": "  Mixed-Case  ",
            "password": format!("  {PASSWORD}  "),
        }))
        .await;

    response.assert_status_ok();
    let body = response_body(&response);
    assert_eq!(body["username"], json!("Mixed-Case"));

    let row: (String, String) =
        sqlx::query_as("SELECT normalized_username, password_hash FROM users INNER JOIN password_credentials ON password_credentials.user_id = users.id")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "mixed-case");
    password_auth::verify_password(PASSWORD, &row.1)
        .expect("the trimmed password should be persisted");
    assert!(password_auth::verify_password(format!("  {PASSWORD}  "), &row.1).is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_rejects_every_invalid_username_shape(pool: SqlitePool) {
    let server = test_server(&pool).await;
    let invalid_usernames = [
        "ab".to_owned(),
        "a".repeat(33),
        "contains space".to_owned(),
        "user.name".to_owned(),
        "usér".to_owned(),
    ];

    for username in invalid_usernames {
        let response = server
            .post(CREATE_USER_URL)
            .json(&json!({ "username": username, "password": PASSWORD }))
            .await;

        response.assert_status_bad_request();
        assert_auth_error(&response, "bad_request", "invalid username");
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_rejects_passwords_outside_length_limits(pool: SqlitePool) {
    let server = test_server(&pool).await;

    for password in ["x".repeat(11), "x".repeat(257)] {
        let response = server
            .post(CREATE_USER_URL)
            .json(&json!({ "username": "valid-user", "password": password }))
            .await;

        response.assert_status_bad_request();
        assert_auth_error(&response, "bad_request", "invalid password");
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_rejects_case_insensitive_duplicate_username(pool: SqlitePool) {
    insert_user(&pool, "ExistingUser", "existinguser", PASSWORD).await;
    let server = test_server(&pool).await;

    let response = server
        .post(CREATE_USER_URL)
        .json(&json!({
            "username": "  EXISTINGUSER  ",
            "password": "another-valid-password",
        }))
        .await;

    response.assert_status_conflict();
    assert_auth_error(&response, "conflict", "username is already taken");

    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM users), (SELECT COUNT(*) FROM password_credentials)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_rejects_invalid_json_requests(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let malformed = server
        .post(CREATE_USER_URL)
        .text(r#"{"username":"alice","password":}"#)
        .content_type("application/json")
        .await;
    malformed.assert_status_bad_request();

    let missing_field = server
        .post(CREATE_USER_URL)
        .json(&json!({ "username": "alice" }))
        .await;
    missing_field.assert_status_unprocessable_entity();

    let wrong_content_type = server
        .post(CREATE_USER_URL)
        .text(r#"{"username":"alice","password":"a-valid-password"}"#)
        .await;
    wrong_content_type.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_user_returns_500_on_database_failure(pool: SqlitePool) {
    let server = test_server(&pool).await;
    pool.close().await;

    let response = server
        .post(CREATE_USER_URL)
        .json(&json!({ "username": "alice", "password": PASSWORD }))
        .await;

    response.assert_status_internal_server_error();
    assert_auth_error(
        &response,
        "internal_server_error",
        "an unexpected error occured",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn get_user_returns_public_user_using_normalized_path_username(pool: SqlitePool) {
    let external_id = insert_user(&pool, "Alice_01", "alice_01", PASSWORD).await;
    let server = test_server(&pool).await;

    let response = server.get("/users/%20ALICE_01%20").await;

    response.assert_status_ok();
    assert_eq!(
        response_body(&response),
        json!({ "external_id": external_id, "username": "Alice_01" })
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn get_user_returns_404_when_username_does_not_exist(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let response = server.get("/users/no-such-user").await;

    response.assert_status_not_found();
    assert_auth_error(&response, "not_found", "user not found");
}

#[sqlx::test(migrations = "./migrations")]
async fn get_user_returns_500_on_database_failure(pool: SqlitePool) {
    let server = test_server(&pool).await;
    pool.close().await;

    let response = server.get("/users/alice").await;

    response.assert_status_internal_server_error();
    assert_auth_error(
        &response,
        "internal_server_error",
        "an unexpected error occured",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn login_creates_session_and_me_returns_current_user(pool: SqlitePool) {
    let external_id = insert_user(&pool, "Alice_01", "alice_01", PASSWORD).await;
    let server = test_server(&pool).await;

    let login_response = server
        .post(LOGIN_URL)
        .form(&json!({
            "username": "  ALICE_01  ",
            "password": PASSWORD,
        }))
        .await;

    login_response.assert_status_no_content();
    assert!(login_response.as_bytes().is_empty());
    let session_cookie = login_response.cookie(SESSION_COOKIE_NAME);
    assert!(!session_cookie.value().is_empty());
    assert!(session_cookie.http_only().unwrap_or(false));

    let me_response = server.get(ME_URL).await;
    me_response.assert_status_ok();
    assert_eq!(
        response_body(&me_response),
        json!({ "external_id": external_id, "username": "Alice_01" })
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_wrong_password_without_creating_session(pool: SqlitePool) {
    insert_user(&pool, "alice", "alice", PASSWORD).await;
    let server = test_server(&pool).await;

    let response = server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice", "password": "wrong-password" }))
        .await;

    response.assert_status_unauthorized();
    assert_auth_error(
        &response,
        "invalid_credentials",
        "invalid username or password",
    );
    assert!(response.maybe_cookie(SESSION_COOKIE_NAME).is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_unknown_username_without_revealing_which_field_failed(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let response = server
        .post(LOGIN_URL)
        .form(&json!({ "username": "unknown", "password": PASSWORD }))
        .await;

    response.assert_status_unauthorized();
    assert_auth_error(
        &response,
        "invalid_credentials",
        "invalid username or password",
    );
    assert!(response.maybe_cookie(SESSION_COOKIE_NAME).is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_malformed_form_requests(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let missing_field = server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice" }))
        .await;
    missing_field.assert_status_unprocessable_entity();

    let wrong_content_type = server
        .post(LOGIN_URL)
        .json(&json!({ "username": "alice", "password": PASSWORD }))
        .await;
    wrong_content_type.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_returns_500_on_database_failure(pool: SqlitePool) {
    let server = test_server(&pool).await;
    pool.close().await;

    let response = server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice", "password": PASSWORD }))
        .await;

    response.assert_status_internal_server_error();
    assert_auth_error(
        &response,
        "internal_server_error",
        "an unexpected error occured",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn login_returns_500_when_stored_password_hash_is_invalid(pool: SqlitePool) {
    insert_user_with_hash(&pool, "alice", "alice", "not-a-password-hash").await;
    let server = test_server(&pool).await;

    let response = server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice", "password": PASSWORD }))
        .await;

    response.assert_status_internal_server_error();
    assert_auth_error(
        &response,
        "internal_server_error",
        "an unexpected error occured",
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn me_redirects_unauthenticated_requests_to_login(pool: SqlitePool) {
    let server = test_server(&pool).await;

    let response = server.get(ME_URL).await;

    response.assert_status(StatusCode::TEMPORARY_REDIRECT);
    response.assert_header(header::LOCATION, "/login?next=%2Fme");
}

#[sqlx::test(migrations = "./migrations")]
async fn me_rejects_session_after_user_is_deleted(pool: SqlitePool) {
    insert_user(&pool, "alice", "alice", PASSWORD).await;
    let server = test_server(&pool).await;

    server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice", "password": PASSWORD }))
        .await
        .assert_status_no_content();

    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .unwrap();

    let response = server.get(ME_URL).await;
    response.assert_status(StatusCode::TEMPORARY_REDIRECT);
    response.assert_header(header::LOCATION, "/login?next=%2Fme");
}

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_500_when_authenticated_user_cannot_be_loaded(pool: SqlitePool) {
    insert_user(&pool, "alice", "alice", PASSWORD).await;
    let server = test_server(&pool).await;

    server
        .post(LOGIN_URL)
        .form(&json!({ "username": "alice", "password": PASSWORD }))
        .await
        .assert_status_no_content();

    pool.close().await;

    let response = server.get(ME_URL).await;
    response.assert_status_internal_server_error();
    assert_eq!(response_body_or_null(&response), Value::Null);
}

fn response_body_or_null(response: &axum_test::TestResponse) -> Value {
    if response.as_bytes().is_empty() {
        Value::Null
    } else {
        response.json::<Value>()
    }
}
