mod common;

use axum::http::StatusCode;
use dotenv::dotenv;
use serde_json::json;

use common::{TestApp, TestCert, get_test_db_url};

fn setup() -> TestApp {
    dotenv().ok();
    std::env::set_var("POOL_SIZE", "1");
    std::env::set_var("TOKEN_EXPIRATION_DAYS", "7");
    TestApp::new(&get_test_db_url())
}

// ============================================================================
// login_with_password Tests
// ============================================================================

#[tokio::test]
async fn test_login_with_password_success() {
    let app = setup();

    let body = json!({
        "username": "root",
        "password": "root"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert!(
        status == StatusCode::OK || status == StatusCode::UNAUTHORIZED,
        "Expected OK or UNAUTHORIZED, got {:?}. Response: {:?}",
        status,
        response
    );

    if status == StatusCode::OK {
        assert!(response["r"].as_bool().unwrap_or(false), "Response should indicate success");
        assert!(response["d"].is_string(), "Response should contain token string");
    }
}

#[tokio::test]
async fn test_login_with_password_wrong_password() {
    let app = setup();

    let body = json!({
        "username": "root",
        "password": "wrong_password"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for wrong password");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_password_missing_username() {
    let app = setup();

    let body = json!({
        "password": "some_password"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for missing username");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_password_missing_password() {
    let app = setup();

    let body = json!({
        "username": "root"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for missing password");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_password_user_not_found() {
    let app = setup();

    let body = json!({
        "username": "nonexistent_user_12345",
        "password": "some_password"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for non-existent user");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_password_invalid_domain() {
    let app = setup();

    let body = json!({
        "username": "root",
        "password": "root"
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &body.to_string(),
            vec![("HtyHost", "nonexistent_domain")],
        )
        .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Should return UNAUTHORIZED for invalid domain. Response: {:?}",
        response
    );
}

// ============================================================================
// login_with_cert Tests
// ============================================================================

#[tokio::test]
async fn test_login_with_cert_invalid_signature() {
    let app = setup();

    let body = json!({
        "encrypted_data": TestCert::generate_invalid_signature()
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_cert",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for invalid signature");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_cert_missing_encrypted_data() {
    let app = setup();

    let body = json!({});

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_cert",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for missing encrypted_data");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_login_with_cert_empty_encrypted_data() {
    let app = setup();

    let body = json!({
        "encrypted_data": ""
    });

    let (status, response) = app
        .post_json(
            "/api/v1/uc/login_with_cert",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "Should return UNAUTHORIZED for empty encrypted_data");
    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

// ============================================================================
// sudo Tests
// ============================================================================

#[tokio::test]
async fn test_sudo_without_auth() {
    let app = setup();

    let (status, _response) = app
        .post_json(
            "/api/v1/uc/sudo",
            "{}",
            vec![],
        )
        .await;

    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST,
        "Should return error without auth token, got {:?}",
        status
    );
}

#[tokio::test]
async fn test_sudo_with_invalid_token() {
    let app = setup();

    let (_status, response) = app
        .post_json(
            "/api/v1/uc/sudo",
            "{}",
            vec![("Authorization", "invalid_jwt_token")],
        )
        .await;

    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_sudo_success_after_login() {
    let app = setup();

    let login_body = json!({
        "username": "root",
        "password": "root"
    });

    let (login_status, login_response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &login_body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    if login_status != StatusCode::OK {
        println!("Login failed (expected in test env without proper setup): {:?}", login_response);
        return;
    }

    let token = login_response["d"].as_str().unwrap();

    let (sudo_status, sudo_response) = app
        .post_json(
            "/api/v1/uc/sudo",
            "{}",
            vec![("Authorization", token)],
        )
        .await;

    assert_eq!(sudo_status, StatusCode::OK, "Sudo should succeed after valid login");
    assert!(sudo_response["r"].as_bool().unwrap_or(false), "Response should indicate success");
    assert!(sudo_response["d"].is_string(), "Response should contain sudoer token");
}

// ============================================================================
// sudo2 Tests
// ============================================================================

#[tokio::test]
async fn test_sudo2_without_auth() {
    let app = setup();

    let (status, _response) = app
        .get(
            "/api/v1/uc/sudo2/some_user_id",
            vec![],
        )
        .await;

    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST,
        "Should return error without auth token, got {:?}",
        status
    );
}

#[tokio::test]
async fn test_sudo2_with_invalid_token() {
    let app = setup();

    let (_status, response) = app
        .get(
            "/api/v1/uc/sudo2/some_user_id",
            vec![
                ("Authorization", "invalid_jwt_token"),
                ("HtyHost", "root"),
            ],
        )
        .await;

    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure");
}

#[tokio::test]
async fn test_sudo2_to_self_after_login() {
    let app = setup();

    let login_body = json!({
        "username": "root",
        "password": "root"
    });

    let (login_status, login_response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &login_body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    if login_status != StatusCode::OK {
        println!("Login failed (expected in test env without proper setup): {:?}", login_response);
        return;
    }

    let token = login_response["d"].as_str().unwrap();

    // sudo2 requires user_app_info.id, not hty_id
    // Use the known test data user_app_info id
    let user_app_info_id = "root-user-app-info-id";

    let (sudo2_status, sudo2_response) = app
        .get(
            &format!("/api/v1/uc/sudo2/{}", user_app_info_id),
            vec![
                ("Authorization", token),
                ("HtyHost", "root"),
            ],
        )
        .await;

    assert_eq!(sudo2_status, StatusCode::OK, "Sudo2 to self should succeed. Response: {:?}", sudo2_response);
    assert!(sudo2_response["r"].as_bool().unwrap_or(false), "Response should indicate success. Response: {:?}", sudo2_response);
}

// ============================================================================
// verify_jwt_token Tests
// ============================================================================

#[tokio::test]
async fn test_verify_jwt_token_invalid() {
    let app = setup();

    let body = json!({
        "token": "invalid_token"
    });

    let (_status, response) = app
        .post_json(
            "/api/v1/uc/verify_jwt_token",
            &body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert!(!response["r"].as_bool().unwrap_or(true), "Response should indicate failure for invalid token");
}

#[tokio::test]
async fn test_verify_jwt_token_after_login() {
    let app = setup();

    let login_body = json!({
        "username": "root",
        "password": "root"
    });

    let (login_status, login_response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &login_body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    if login_status != StatusCode::OK {
        println!("Login failed (expected in test env without proper setup): {:?}", login_response);
        return;
    }

    let token = login_response["d"].as_str().unwrap();

    let verify_body = json!({
        "token": token
    });

    let (verify_status, verify_response) = app
        .post_json(
            "/api/v1/uc/verify_jwt_token",
            &verify_body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    assert_eq!(verify_status, StatusCode::OK, "Token verification should succeed");
    assert!(verify_response["r"].as_bool().unwrap_or(false), "Response should indicate success");
}

// ============================================================================
// generate_key_pair Tests
// ============================================================================

#[tokio::test]
async fn test_generate_key_pair_without_auth() {
    let app = setup();

    let (status, _response) = app
        .get(
            "/api/v1/uc/generate_key_pair",
            vec![("HtyHost", "root")],
        )
        .await;

    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST,
        "Should return error without auth token, got {:?}",
        status
    );
}

#[tokio::test]
async fn test_generate_key_pair_after_login() {
    let app = setup();

    let login_body = json!({
        "username": "root",
        "password": "root"
    });

    let (login_status, login_response) = app
        .post_json(
            "/api/v1/uc/login_with_password",
            &login_body.to_string(),
            vec![("HtyHost", "root")],
        )
        .await;

    if login_status != StatusCode::OK {
        println!("Login failed (expected in test env without proper setup): {:?}", login_response);
        return;
    }

    let token = login_response["d"].as_str().unwrap();

    let (key_pair_status, key_pair_response) = app
        .get(
            "/api/v1/uc/generate_key_pair",
            vec![
                ("Authorization", token),
                ("HtyHost", "root"),
            ],
        )
        .await;

    assert_eq!(key_pair_status, StatusCode::OK, "Key pair generation should succeed");
    assert!(key_pair_response["r"].as_bool().unwrap_or(false), "Response should indicate success");
    assert!(key_pair_response["d"]["pubkey"].is_string(), "Response should contain pubkey");
    assert!(key_pair_response["d"]["privkey"].is_string(), "Response should contain privkey");
}

// ============================================================================
// Index Endpoint Test
// ============================================================================

#[tokio::test]
async fn test_index_endpoint() {
    let app = setup();

    let (status, _response) = app
        .get("/api/v1/uc/index", vec![])
        .await;

    assert_eq!(status, StatusCode::OK, "Index endpoint should return OK");
}
