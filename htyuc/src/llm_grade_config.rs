//! Per-teacher LLM auto-grading configuration (API key + intro + model).

use std::ops::DerefMut;
use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use axum_macros::debug_handler;
use htycommons::common::{HtyErr, HtyErrCode, HtyResponse};
use htycommons::jwt::jwt_decode_token;
use htycommons::models::MultiVals;
use htycommons::secret_box::{mask_secret, open_secret, seal_secret};
use htycommons::web::{
    wrap_json_anyhow_err, wrap_json_ok_resp, AuthorizationHeader, HtyHostHeader, HtySudoerTokenHeader,
};
use htyuc_models::models::{HtyUser, UserSetting};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{extract_conn, fetch_db_conn, DbState};

pub const LLM_GRADE_CONFIG_SETTING_KEY: &str = "llm_grade_config";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmGradeConfigStored {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub api_key_enc: Option<String>,
    #[serde(default)]
    pub grading_self_intro: Option<String>,
    #[serde(default)]
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReqSetLlmGradeConfig {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub grading_self_intro: Option<String>,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespLlmGradeConfig {
    pub enabled: bool,
    pub key_masked: Option<String>,
    pub grading_self_intro: Option<String>,
    pub llm_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqLlmGradeConfigInternal {
    pub hty_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespLlmGradeConfigInternal {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub grading_self_intro: Option<String>,
    pub llm_model: Option<String>,
}

fn parse_stored(settings: &Option<MultiVals<UserSetting>>) -> LlmGradeConfigStored {
    let all = settings
        .as_ref()
        .and_then(|m| m.vals.as_ref())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    for s in all {
        if s.k.as_deref() == Some(LLM_GRADE_CONFIG_SETTING_KEY) {
            if let Some(v) = &s.v {
                if let Ok(parsed) = serde_json::from_str::<LlmGradeConfigStored>(v) {
                    return parsed;
                }
            }
        }
    }
    LlmGradeConfigStored::default()
}

fn upsert_setting(user: &mut HtyUser, stored: &LlmGradeConfigStored) -> anyhow::Result<()> {
    let json = serde_json::to_string(stored)?;
    let mut settings = user.settings.clone().unwrap_or_default();
    let vals = settings.vals.get_or_insert_with(Vec::new);
    if let Some(existing) = vals
        .iter_mut()
        .find(|s| s.k.as_deref() == Some(LLM_GRADE_CONFIG_SETTING_KEY))
    {
        existing.v = Some(json);
    } else {
        vals.push(UserSetting {
            k: Some(LLM_GRADE_CONFIG_SETTING_KEY.to_string()),
            v: Some(json),
            app_id: None,
            role_key: None,
        });
    }
    user.settings = Some(settings);
    Ok(())
}

fn to_public_resp(stored: &LlmGradeConfigStored) -> RespLlmGradeConfig {
    let key_masked = stored.api_key_enc.as_ref().and_then(|enc| {
        open_secret(enc)
            .ok()
            .map(|plain| mask_secret(&plain))
    });
    RespLlmGradeConfig {
        enabled: stored.enabled.unwrap_or(false),
        key_masked,
        grading_self_intro: stored.grading_self_intro.clone(),
        llm_model: stored
            .llm_model
            .clone()
            .or_else(|| Some("dashscope/qwen3-omni-flash".to_string())),
    }
}

pub fn raw_get_llm_grade_config_for_user(user: &HtyUser) -> RespLlmGradeConfig {
    to_public_resp(&parse_stored(&user.settings))
}

pub fn raw_get_llm_grade_config_internal(user: &HtyUser) -> anyhow::Result<RespLlmGradeConfigInternal> {
    let stored = parse_stored(&user.settings);
    let api_key = stored
        .api_key_enc
        .as_ref()
        .map(|enc| open_secret(enc))
        .transpose()?;
    Ok(RespLlmGradeConfigInternal {
        enabled: stored.enabled.unwrap_or(false),
        api_key,
        grading_self_intro: stored.grading_self_intro.clone(),
        llm_model: stored
            .llm_model
            .clone()
            .or_else(|| Some("dashscope/qwen3-omni-flash".to_string())),
    })
}

#[debug_handler]
pub async fn set_llm_grade_config(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req): Json<ReqSetLlmGradeConfig>,
) -> Json<HtyResponse<RespLlmGradeConfig>> {
    debug!("set_llm_grade_config -> starts");
    match raw_set_llm_grade_config(&auth, &req, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("set_llm_grade_config -> {e}");
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_set_llm_grade_config(
    auth: &AuthorizationHeader,
    req: &ReqSetLlmGradeConfig,
    db_pool: Arc<DbState>,
) -> anyhow::Result<RespLlmGradeConfig> {
    let hty_id = jwt_decode_token(auth)?
        .hty_id
        .ok_or_else(|| anyhow::anyhow!("hty_id required"))?;
    let mut conn = extract_conn(fetch_db_conn(&db_pool)?);
    let mut user = HtyUser::find_by_hty_id(&hty_id, conn.deref_mut())?;
    let mut stored = parse_stored(&user.settings);
    if let Some(enabled) = req.enabled {
        stored.enabled = Some(enabled);
    }
    if let Some(intro) = &req.grading_self_intro {
        stored.grading_self_intro = Some(intro.clone());
    }
    if let Some(model) = &req.llm_model {
        stored.llm_model = Some(model.clone());
    }
    if let Some(key) = &req.api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            stored.api_key_enc = Some(seal_secret(trimmed)?);
        }
    }
    upsert_setting(&mut user, &stored)?;
    HtyUser::update(&user, conn.deref_mut())?;
    Ok(to_public_resp(&stored))
}

#[debug_handler]
pub async fn get_llm_grade_config(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<RespLlmGradeConfig>> {
    match raw_get_llm_grade_config(auth, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

async fn raw_get_llm_grade_config(
    auth: AuthorizationHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<RespLlmGradeConfig> {
    let hty_id = jwt_decode_token(&auth)?
        .hty_id
        .ok_or_else(|| anyhow::anyhow!("hty_id required"))?;
    let user = HtyUser::find_by_hty_id(
        &hty_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(raw_get_llm_grade_config_for_user(&user))
}

#[debug_handler]
pub async fn get_llm_grade_config_internal(
    _host: HtyHostHeader,
    root: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req): Json<ReqLlmGradeConfigInternal>,
) -> Json<HtyResponse<RespLlmGradeConfigInternal>> {
    if root.0.is_empty() {
        return wrap_json_anyhow_err(anyhow::anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("sudo token required".into()),
        }));
    }
    let conn_result = fetch_db_conn(&db_pool);
    let conn = match conn_result {
        Ok(c) => c,
        Err(e) => return wrap_json_anyhow_err(e),
    };
    match HtyUser::find_by_hty_id(&req.hty_id, extract_conn(conn).deref_mut()) {
        Ok(user) => match raw_get_llm_grade_config_internal(&user) {
            Ok(d) => wrap_json_ok_resp(d),
            Err(e) => wrap_json_anyhow_err(e),
        },
        Err(e) => wrap_json_anyhow_err(e),
    }
}
