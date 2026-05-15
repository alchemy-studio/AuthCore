use axum::{Json, extract::State, http::StatusCode};
use htycommons::common::{current_local_datetime, HtyResponse, HtyErr, HtyErrCode};
use htycommons::db::{extract_conn, fetch_db_conn, DbState};
use htycommons::web::{HtyToken, HtySudoerTokenHeader, wrap_json_ok_resp};
use diesel::{sql_query, RunQueryDsl, QueryableByName};
use std::ops::DerefMut;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

fn generate_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let mut seed = nanos;
    (0..8).map(|_| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = (seed >> 33) as usize % CHARSET.len();
        CHARSET[idx] as char
    }).collect()
}

fn new_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("inv{:016x}", ts)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchGenerateReq {
    pub count: Option<i32>,
    pub org_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConsumeReq {
    pub code: String,
}

/// POST /api/v1/uc/invite_code/batch
pub async fn batch_generate(
    State(db_pool): State<Arc<DbState>>,
    token: HtyToken,
    Json(req): Json<BatchGenerateReq>,
) -> Result<Json<HtyResponse<Vec<String>>>, StatusCode> {
    let count = req.count.unwrap_or(10).max(1).min(100);
    let teacher_hty_id = token.hty_id.ok_or(StatusCode::BAD_REQUEST)?;
    let pool = fetch_db_conn(&db_pool).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut guard = extract_conn(pool);
    let conn = &mut *guard;
    let now = current_local_datetime().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut codes = Vec::new();
    for _ in 0..count {
        let code_str = generate_code();
        let id = new_uuid();
        let org_val = match &req.org_id {
            Some(o) => format!("'{}'", o.replace('\'', "''")),
            None => "NULL".to_string(),
        };
        let sql = format!(
            "INSERT INTO invitation_codes (id, code, teacher_id, org_id, status, created_at) VALUES ('{}', '{}', '{}', {}, 'active', '{}')",
            id, code_str, teacher_hty_id, org_val, now,
        );
        sql_query(&sql).execute(conn).map_err(|e| {
            error!("[invite_code batch] insert error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        codes.push(code_str);
    }
    Ok(wrap_json_ok_resp(codes))
}

/// POST /api/v1/uc/invite_code/consume
pub async fn consume(
    State(db_pool): State<Arc<DbState>>,
    _sudoer: HtySudoerTokenHeader,
    Json(req): Json<ConsumeReq>,
) -> Result<Json<HtyResponse<serde_json::Value>>, StatusCode> {
    let pool = fetch_db_conn(&db_pool).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut guard = extract_conn(pool);
    let conn = &mut *guard;

    let safe_code = req.code.replace('\'', "''");
    let sql = format!(
        "SELECT code, teacher_id, org_id, status, created_at::text, consumed_at::text FROM invitation_codes WHERE code = '{}' AND status = 'active' LIMIT 1",
        safe_code
    );
    let rows = sql_query(&sql)
        .load::<InviteCodeRow>(conn)
        .map_err(|e| {
            error!("[invite_code consume] query error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match rows.first() {
        Some(row) => {
            let info = serde_json::json!({
                "code": row.code,
                "teacher_id": row.teacher_id,
                "org_id": row.org_id,
                "status": row.status,
                "created_at": row.created_at,
                "consumed_at": row.consumed_at,
            });
            Ok(wrap_json_ok_resp(info))
        }
        None => {
            Ok(Json(HtyResponse {
                r: false, d: None,
                e: Some("invalid or used invite code".to_string()),
                hty_err: Some(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("invite code not found or already used".to_string()),
                }),
            }))
        }
    }
}

/// GET /api/v1/uc/invite_code/list
pub async fn list(
    State(db_pool): State<Arc<DbState>>,
    token: HtyToken,
) -> Result<Json<HtyResponse<Vec<serde_json::Value>>>, StatusCode> {
    let teacher_hty_id = token.hty_id.ok_or(StatusCode::BAD_REQUEST)?;
    let pool = fetch_db_conn(&db_pool).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut guard = extract_conn(pool);
    let conn = &mut *guard;

    let sql = format!(
        "SELECT code, teacher_id, org_id, status, created_at::text, consumed_at::text FROM invitation_codes WHERE teacher_id = '{}' ORDER BY created_at DESC",
        teacher_hty_id,
    );
    let rows = sql_query(&sql)
        .load::<InviteCodeRow>(conn)
        .map_err(|e| {
            error!("[invite_code list] query error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "code": r.code,
            "teacher_id": r.teacher_id,
            "org_id": r.org_id,
            "status": r.status,
            "created_at": r.created_at,
            "consumed_at": r.consumed_at,
        })
    }).collect();

    Ok(wrap_json_ok_resp(items))
}

#[derive(QueryableByName, Debug)]
struct InviteCodeRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    teacher_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    created_at: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    consumed_at: Option<String>,
}
