use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::util::ServiceExt;
use serde_json::Value;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        dotenv::dotenv().ok();

        if std::env::var("UC_DB_URL").is_err() {
            std::env::set_var("UC_DB_URL", "postgres://htyuc:htyuc@localhost:5433/htyuc_test");
        }
        if std::env::var("REDIS_HOST").is_err() {
            std::env::set_var("REDIS_HOST", "localhost");
        }
        if std::env::var("REDIS_PORT").is_err() {
            std::env::set_var("REDIS_PORT", "6380");
        }
        if std::env::var("JWT_KEY").is_err() {
            std::env::set_var("JWT_KEY", "test_jwt_key_for_testing_only_1234567890");
        }
        if std::env::var("POOL_SIZE").is_err() {
            std::env::set_var("POOL_SIZE", "5");
        }
        if std::env::var("EXPIRATION_DAYS").is_err() {
            std::env::set_var("EXPIRATION_DAYS", "7");
        }
        if std::env::var("SKIP_POST_LOGIN").is_err() {
            std::env::set_var("SKIP_POST_LOGIN", "true");
        }
        if std::env::var("SKIP_REGISTRATION").is_err() {
            std::env::set_var("SKIP_REGISTRATION", "true");
        }
    });
}

pub struct TestApp {
    pub router: Router,
}

impl TestApp {
    pub fn new(db_url: &str) -> Self {
        init_test_env();
        let router = htyuc::uc_rocket(db_url);
        Self { router }
    }

    pub async fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: Vec<(&str, &str)>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/json");

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let request = request.body(Body::from(body.to_string())).unwrap();

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

        (status, body)
    }

    pub async fn get(
        &self,
        uri: &str,
        headers: Vec<(&str, &str)>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method("GET")
            .uri(uri);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let request = request.body(Body::empty()).unwrap();

        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

        (status, body)
    }
}

pub struct TestCert;

impl TestCert {
    pub fn generate_invalid_signature() -> String {
        "invalid_signature_data_12345".to_string()
    }
}

pub fn get_test_db_url() -> String {
    init_test_env();
    std::env::var("UC_DB_URL").unwrap_or_else(|_| "postgres://htyuc:htyuc@localhost:5433/htyuc_test".to_string())
}
