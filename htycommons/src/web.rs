use crate::common::{HtyErr, HtyErrCode, HtyResponse, TimeUnit};
use crate::db::CommonMeta;
use anyhow::anyhow;
use axum::extract::FromRequestParts;
use axum::http::header::HOST;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use axum::Router;
use chrono::NaiveDateTime;
use log::debug;
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::net::TcpListener;
// use crate::impl_jsonb_boilerplate;
// use crate::db::SingleVal;
// use log::debug;
use crate::jwt::{jwt_decode_token, jwt_encode_token};
use crate::redis_util::verify_jwt;

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostHeader(pub String);

impl Deref for HostHeader {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> FromRequestParts<B> for HostHeader
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = &req.headers;
        Ok(Self(
            headers[HOST].to_str().map_err(internal_error)?.to_string(),
        ))
    }
}

//------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct UnionIdHeader(pub String);

impl<B> FromRequestParts<B> for UnionIdHeader
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = &req.headers;
        Ok(Self(
            headers["UnionId"]
                .to_str()
                .map_err(internal_error)?
                .to_string(),
        ))
    }
}

impl Deref for UnionIdHeader {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizationHeader(pub String);

impl<B> FromRequestParts<B> for AuthorizationHeader
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = &req.headers;
        if headers.contains_key("Authorization") {
            Ok(Self(
                headers["Authorization"]
                    .to_str()
                    .map_err(internal_error)?
                    .to_string(),
            ))
        } else {
            let resp = wrap_auth_err(&None);
            Err((StatusCode::UNAUTHORIZED, resp))
        }
    }
}

impl<B> FromRequestParts<B> for HtyToken
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = req.headers.clone();

        let token_result = headers["Authorization"].to_str();

        if token_result.is_err() {
            let resp = wrap_auth_err(&None);
            Err((StatusCode::UNAUTHORIZED, resp))
        } else {
            match token_result {
                Ok(token_str) => {
                    match jwt_decode_token(&token_str.to_string()) {
                        Ok(resp) => Ok(resp),
                        Err(err) => {
                            let resp = wrap_auth_err(&Some(err.to_string()));
                            Err((StatusCode::UNAUTHORIZED, resp))
                        }
                    }
                }
                Err(_) => {
                    let resp = wrap_auth_err(&None);
                    Err((StatusCode::UNAUTHORIZED, resp))
                }
            }
        }
    }
}

impl Deref for AuthorizationHeader {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct HtySudoerTokenHeader(pub String);

impl<B> FromRequestParts<B> for HtySudoerTokenHeader
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);
    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = &req.headers;
        if headers.contains_key("HtySudoerToken") {
            let sudoer_token = headers["HtySudoerToken"]
                .to_str()
                .map_err(internal_error)?
                .to_string();
            match verify_jwt(&sudoer_token) {
                Ok(()) => {
                    debug!("verify sudoer token okay! -> {:?}", sudoer_token);
                    Ok(Self(sudoer_token))
                }
                Err(e) => {
                    debug!(
                        "verify sudoer token error! -> TOKEN: {:?} / ERR: {:?}",
                        sudoer_token, e
                    );
                    let resp = wrap_sudo_err(&Some(sudoer_token));
                    Err((StatusCode::UNAUTHORIZED, resp))
                }
            }
        } else {
            let resp = wrap_sudo_err(&None);
            Err((StatusCode::UNAUTHORIZED, resp))
        }
    }
}

impl Deref for HtySudoerTokenHeader {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct HtyHostHeader(pub String);

impl<B> FromRequestParts<B> for HtyHostHeader
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, _state: &B) -> Result<Self, Self::Rejection> {
        let headers = &req.headers;
        Ok(Self(
            headers["HtyHost"]
                .to_str()
                .map_err(internal_error)?
                .to_string(),
        ))
    }
}

impl Deref for HtyHostHeader {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ------

pub fn wrap_err_msg(err_type: HtyErrCode, err_message: &String) -> HtyErr {
    HtyErr {
        code: err_type,
        reason: Some(err_message.clone()),
    }
}

pub fn wrap_err(err_type: HtyErrCode, err: Box<dyn Error>) -> HtyErr {
    HtyErr {
        code: err_type,
        reason: Some(err.to_string()),
    }
}

pub fn wrap_json_hty_err<T: Serialize + DeserializeOwned + Debug + Clone>(
    err: HtyErr,
) -> Json<HtyResponse<T>> {
    Json(wrap_hty_err(err))
}

pub fn wrap_json_anyhow_err<T: Serialize + DeserializeOwned + Debug + Clone>(
    err: anyhow::Error,
) -> Json<HtyResponse<T>> {
    Json(wrap_anyhow_err(err))
}

pub fn wrap_hty_err<T: Serialize + DeserializeOwned + Debug + Clone>(
    err: HtyErr,
) -> HtyResponse<T> {
    HtyResponse {
        r: false,
        d: None,
        e: Some(err.to_string()),
        hty_err: Some(err.clone()),
    }
}

pub fn wrap_auth_err(some_err_str: &Option<String>) -> String {
    serde_json::to_string(&HtyResponse {
        r: false,
        d: some_err_str.clone(),
        e: Some("AuthorizationErr".to_string()),
        hty_err: Some(HtyErr {
            code: HtyErrCode::InternalErr,
            reason: None,
        }),
    })
    .unwrap_or_else(|_| {
        // 如果序列化失败，尝试序列化一个简化的响应
        serde_json::to_string(&HtyResponse::<String> {
            r: false,
            d: None,
            e: Some("AuthorizationErr".to_string()),
            hty_err: Some(HtyErr {
                code: HtyErrCode::InternalErr,
                reason: None,
            }),
        })
        .expect("Failed to serialize simplified HtyResponse")
    })
}

pub fn wrap_sudo_err(some_token: &Option<String>) -> String {
    serde_json::to_string(&HtyResponse {
        r: false,
        d: some_token.clone(),
        e: Some("HtySudoerTokenErr".to_string()),
        hty_err: Some(HtyErr {
            code: HtyErrCode::AuthenticationFailed,
            reason: Some("HtySudoerTokenErr".to_string()),
        }),
    })
    .unwrap_or_else(|_| {
        // 如果序列化失败，尝试序列化一个简化的响应
        serde_json::to_string(&HtyResponse::<String> {
            r: false,
            d: None,
            e: Some("HtySudoerTokenErr".to_string()),
            hty_err: Some(HtyErr {
                code: HtyErrCode::AuthenticationFailed,
                reason: Some("HtySudoerTokenErr".to_string()),
            }),
        })
        .expect("Failed to serialize simplified HtyResponse")
    })
}

pub fn wrap_anyhow_err<T: Serialize + DeserializeOwned + Debug + Clone>(
    err: anyhow::Error,
) -> HtyResponse<T> {
    HtyResponse {
        r: false,
        d: None,
        e: Some(err.to_string()),
        hty_err: Some(HtyErr {
            code: HtyErrCode::InternalErr,
            reason: Some(err.to_string()),
        }),
    }
}

pub fn wrap_json_ok_resp<T: Serialize + DeserializeOwned + Debug + Clone>(
    ok: T,
) -> Json<HtyResponse<T>> {
    Json(wrap_ok_resp(ok))
}

pub fn wrap_ok_resp<T: Serialize + DeserializeOwned + Debug + Clone>(ok: T) -> HtyResponse<T> {
    HtyResponse {
        r: true,
        d: Some(ok),
        e: None,
        hty_err: None,
    }
}

//------------------------------------------------

pub async fn launch_rocket(port: u16, app: Router) -> anyhow::Result<()> {
    debug!("launching rocket...🚀");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    // tracing::debug!("listening on {}", addr);
    debug!("listening on {}", addr);
    // https://tokio.rs/blog/2023-11-27-announcing-axum-0-7-0
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| anyhow!("Failed to bind to address: {}", e))?;
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| anyhow!("Failed to start server: {}", e))?;
    Ok(())
}


//------------------------------------------------

pub fn set_uc_url(url: &String) {
    env::set_var("UC_URL", url.clone());
}

pub fn get_uc_url() -> String {
    env::var("UC_URL").expect("UC_URL not set!")
}

pub fn get_ngx_url() -> String {
    env::var("NGX_URL").expect("NGX_URL not set!")
}

pub fn set_ws_url(url: &String) {
    env::set_var("WS_URL", url.clone());
}

pub fn get_ws_url() -> String {
    env::var("WS_URL").expect("WS_URL not set!")
}

pub fn get_music_room_mini_url() -> String {
    env::var("MUSIC_ROOM_MINI_URL").expect("MUSIC_ROOM_MINI_URL not set!")
}

pub fn random_port() -> u16 {
    rand::rng().random_range(10000..20000)
}

pub fn get_uc_port() -> anyhow::Result<u16> {
    env::var("UC_PORT")
        .expect("UC_PORT not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse UC_PORT: {}", e))
}

pub fn generate_ports() {
    let mut rng = rand::rng();
    let uc_port = rng.random_range(10000..20000);
    let mut ws_port = rng.random_range(10000..20000);

    while ws_port == uc_port {
        ws_port = rng.random_range(10000..20000);
    }

    set_uc_port(uc_port);
    set_ws_port(ws_port);
}

pub fn set_uc_port(port: u16) {
    env::set_var("UC_PORT", port.to_string());
}

pub fn get_ws_port() -> anyhow::Result<u16> {
    env::var("WS_PORT")
        .expect("WS_PORT not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse WS_PORT: {}", e))
}

pub fn set_ws_port(port: u16) {
    env::set_var("WS_PORT", port.to_string());
}
pub fn get_kc_port() -> anyhow::Result<u16> {
    env::var("KC_PORT")
        .expect("KC_PORT not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse KC_PORT: {}", e))
}

pub fn set_kc_port(port: u16) {
    env::set_var("KC_PORT", port.to_string());
}

pub fn skip_post_login() -> anyhow::Result<bool> {
    env::var("SKIP_POST_LOGIN")
        .expect("SKIP_POST_LOGIN not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse SKIP_POST_LOGIN: {}", e))
}

pub fn skip_post_registration() -> anyhow::Result<bool> {
    env::var("SKIP_REGISTRATION")
        .expect("SKIP_REGISTRATION not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse SKIP_REGISTRATION: {}", e))
}

pub fn skip_wx_push() -> anyhow::Result<bool> {
    env::var("SKIP_WX_PUSH")
        .expect("SKIP_WX_PUSH not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse SKIP_WX_PUSH: {}", e))
}

pub fn get_domain() -> String {
    env::var("DOMAIN")
        .expect("DOMAIN not set!!!")
        .to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHtyRole {
    pub hty_role_id: Option<String>,
    pub user_app_info_id: Option<String>,
    pub app_ids: Option<Vec<String>>,
    pub role_key: Option<String>,
    pub role_desc: Option<String>,
    pub role_status: Option<String>,
    pub labels: Option<Vec<ReqHtyLabel>>,
    pub actions: Option<Vec<ReqHtyAction>>,
    pub style: Option<String>,
    pub role_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHtyAction {
    pub hty_action_id: Option<String>,
    pub action_name: Option<String>,
    pub action_desc: Option<String>,
    pub action_status: Option<String>,
    pub roles: Option<Vec<ReqHtyRole>>,
    pub labels: Option<Vec<ReqHtyLabel>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHtyLabel {
    pub hty_label_id: Option<String>,
    pub label_name: Option<String>,
    pub label_desc: Option<String>,
    pub label_status: Option<String>,
    pub roles: Option<Vec<ReqHtyRole>>,
    pub actions: Option<Vec<ReqHtyAction>>,
    pub style: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHtyTag {
    pub tag_id: Option<String>,
    pub tag_name: Option<String>,
    pub tag_desc: Option<String>,
    pub style: Option<String>,
    pub refs: Option<Vec<ReqHtyTagRef>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqHtyTagRef {
    pub tag_ref_id: Option<String>, // ref这张表自己的id
    pub hty_tag_id: Option<String>, // tag这张表的id，和下面`tag->tag_id`冗余
    pub ref_id: Option<String>,     // 引用表id
    pub ref_type: Option<String>,
    pub meta: Option<CommonMeta>,
    pub tag: Option<ReqHtyTag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqTagRefsByRefId {
    // ref_id可以对应多个`ReqHtyTagRef`
    pub ref_id: Option<String>,
    pub ref_type: Option<String>,
    pub tag_refs: Option<Vec<ReqHtyTagRef>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HtyToken {
    pub token_id: String,
    pub hty_id: Option<String>,
    pub app_id: Option<String>,
    pub ts: NaiveDateTime,
    // generated time
    pub roles: Option<Vec<ReqHtyRole>>,
    pub tags: Option<Vec<ReqHtyTag>>,
}

impl HtyToken {
    pub fn from(json: &str) -> serde_json::Result<HtyToken> {
        serde_json::from_str::<HtyToken>(json)
    }

    pub fn from_jwt(jwt_str: &str) -> anyhow::Result<HtyToken> {
        match jwt_decode_token(&(jwt_str.to_string())) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn to_jwt(&self) -> anyhow::Result<String> {
        match jwt_encode_token(self.clone()) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(anyhow!(err)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqKV {
    pub key: Option<String>,
    pub val: Option<String>,
    pub exp: Option<i32>,
    pub exp_unit: Option<TimeUnit>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqTeacherStudent {
    pub id: Option<String>,
    pub teacher_id: Option<String>,
    pub student_id: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ReqDateFilter {
//     pub start_date: Option<NaiveDateTime>,
//     pub end_date: Option<NaiveDateTime>,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqTeacherStudentsQuery {
    pub hty_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqCommonQuery {
    pub vals: Option<Vec<String>>,
}

