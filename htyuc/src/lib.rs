extern crate dotenv;
extern crate htycommons;
extern crate serde_derive;

use axum::routing::post;

use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use diesel::PgConnection;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use tokio::task;
use tower_http::trace::TraceLayer;

use htyuc_models::models::*;
use htyuc_models::wx::{find_wx_openid_by_unionid_and_hty_app, get_jsapi_ticket, get_or_save_wx_access_token, get_union_id_by_auth_code, push_wx_message, refresh_cache_and_get_wx_all_follower_openids};
use htycommons::cert::{generate_cert_key_pair, verify, HtyKeyPair};
use htycommons::common::{current_local_datetime, get_page_and_page_size, get_some_from_query_params, HtyErr, HtyErrCode, HtyResponse, APP_STATUS_ACTIVE, APP_STATUS_DELETED, TimeUnit, extract_filename_from_url};
use htycommons::db::{exec_read_write_task, extract_conn, fetch_db_conn, pool, DbState};
use htycommons::jwt::{jwt_decode_token, jwt_encode_token};
use htycommons::logger::debug;
use htycommons::redis_util::*;
use htycommons::upyun::{generate_upyun_token, get_upyun_operator, get_upyun_password, upyun_delete_by_filename};
use htycommons::web::{get_uc_url, skip_post_login, skip_post_registration, wrap_json_anyhow_err, wrap_json_hty_err, wrap_json_ok_resp, AuthorizationHeader, HtyHostHeader, HtySudoerTokenHeader, HtyToken, ReqHtyAction, ReqHtyLabel, ReqHtyRole, ReqHtyTag, ReqHtyTagRef, ReqTagRefsByRefId, UnionIdHeader, ReqKV};
use htycommons::wx::{code2session, WxId, WxLogin, WxParams, WxSession};
use htycommons::{db, uuid};

use tracing::{debug, warn, error};
use htycommons::models::*;

pub mod ddl;
pub mod r_uc;
mod notifications;
pub mod test_scaffold;

// pub mod wx;

async fn index() -> &'static str {
    "-=HTYUC=-"
}

#[allow(dead_code)]
fn find_all_unregisted_users_by_app_key() {
    unimplemented!()
}

#[allow(dead_code)]
async fn update_user_info_by_wx_app() {
    unimplemented!()
}

async fn del_cached_kv(_sudoer: HtySudoerTokenHeader,
                       _host: HtyHostHeader,
                       Path(key): Path<String>, ) -> Json<HtyResponse<()>> {
    debug!("del_cached_kv -> starts / in data: {:?}", key);
    let mut prefix_key = CACHED.to_string();
    prefix_key.push_str(key.clone().as_str());

    match raw_del_cached_kv(&prefix_key) {
        Ok(_) => {
            debug!("raw_del_cached_kv -> success");
            wrap_json_ok_resp(())
        }
        Err(e) => {
            debug!("raw_del_cached_kv -> err: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_del_cached_kv(key: &String) -> anyhow::Result<Option<()>> {
    debug!("raw_del_cached_kv -> key to delete: {:?}", key);
    del_some_from_redis(&key)
}

async fn save_cached_kv(_sudoer: HtySudoerTokenHeader,
                        _host: HtyHostHeader,
                        Json(in_kv): Json<ReqKV>, ) -> Json<HtyResponse<()>> {
    debug!("save_cached_kv -> starts / in data: {:?}", in_kv);

    match raw_save_cached_kv(&in_kv) {
        Ok(ok) => {
            debug!("raw_save_cached_kv -> success");
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            debug!("raw_save_cached_kv -> err: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_save_cached_kv(in_kv: &ReqKV) -> anyhow::Result<()> {
    let c_in_kv = in_kv.clone();
    let mut prefix_key = CACHED.to_string();
    prefix_key.push_str(c_in_kv.key.unwrap().clone().as_str());

    if in_kv.exp_unit.is_some() {
        match in_kv.exp_unit.clone().unwrap() {
            TimeUnit::SECOND => {
                save_kv_to_redis_with_exp_secs(&prefix_key, &c_in_kv.val.unwrap(), c_in_kv.exp.unwrap_or(7) as usize) // default 7 seconds.
            }
            TimeUnit::MINUTE => {
                save_kv_to_redis_with_exp_minutes(&prefix_key, &c_in_kv.val.unwrap(), c_in_kv.exp.unwrap_or(7) as usize) // default 7 minutes.
            }
            TimeUnit::HOUR => {
                save_kv_to_redis_with_exp_hours(&prefix_key, &c_in_kv.val.unwrap(), c_in_kv.exp.unwrap_or(7) as usize) // default 7 hours.
            }
            TimeUnit::DAY => {
                save_kv_to_redis_with_exp_days(&prefix_key, &c_in_kv.val.unwrap(), c_in_kv.exp.unwrap_or(7) as usize) // default 7 days.
            }
        }
    } else {
        save_kv_to_redis_with_exp_days(&prefix_key, &c_in_kv.val.unwrap(), c_in_kv.exp.unwrap_or(7) as usize) // default 7 days.
    }
}

async fn get_cached_kv(Path(key): Path<String>) -> Json<HtyResponse<Option<String>>> {
    debug!("get_cached_kv -> starts / in key: {:?}", key);
    let mut prefix_key = CACHED.to_string();
    prefix_key.push_str(key.clone().as_str());

    match raw_get_cached_kv(&prefix_key) {
        Ok(ok) => {
            debug!("get_cached_kv -> success / val: {:?}", ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            debug!("get_cached_kv -> err: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_get_cached_kv(key: &String) -> anyhow::Result<Option<String>> {
    get_opt_value_from_redis(key)
}


async fn find_users_by_domain(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqHtyUser>>> {
    debug!("find_users_by_domain -> starts");
    let domain = (*host).clone();
    match HtyUser::all_users_by_app_domain(&domain, extract_conn(conn).deref_mut()) {
        Ok(users) => {
            debug!(
                "find_users_by_domain -> success to find users: {:?}!",
                users
            );
            wrap_json_ok_resp(HtyUser::to_req_users(&users))
        }
        Err(e) => {
            error!("find_users_by_domain -> failed to find users, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn find_hty_template_with_data_by_key_and_app_id(
    _sudoer: HtySudoerTokenHeader,
    Query(params): Query<HashMap<String, String>>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyTemplate<String>>> {
    debug!(
        "find_hty_template_with_data_by_key_and_app_id START params -> {:?}",
        params
    );

    let id_app = get_some_from_query_params::<String>("app_id", &params);
    let key_template = get_some_from_query_params::<String>("template_key", &params);

    if id_app.is_none() || key_template.is_none() {
        return wrap_json_hty_err::<ReqHtyTemplate<String>>(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("id_app or key_template is none".into()),
        });
    }

    match raw_find_hty_template_with_data_by_key_and_app_id(
        &id_app.unwrap(),
        &key_template.unwrap(),
        extract_conn(conn).deref_mut(),
    ) {
        Ok(resp) => {
            debug!(
                "find_hty_template_with_data_by_key_and_app_id -> RESP HtyTemplate: {:?}!",
                resp
            );
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!(
                "find_hty_template_with_data_by_key_and_app_id -> FAILED, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_hty_template_with_data_by_key_and_app_id(
    id_app: &String,
    key_template: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyTemplate<String>> {
    let (template, template_data) =
        HtyTemplateData::<String>::find_with_template_key_and_app_id_with_string(
            key_template,
            id_app,
            conn,
        )?;

    let mut resp_template = ReqHtyTemplate {
        id: Some(template.id.clone()),
        template_key: Some(template.template_key.clone()),
        created_at: Some(template.created_at.clone()),
        created_by: Some(template.created_by.clone()),
        template_desc: template.template_desc.clone(),
        datas: None,
    };

    let resp_template_data = ReqHtyTemplateData {
        id: Some(template_data.id.clone()),
        app_id: Some(template_data.app_id.clone()),
        template_id: Some(template_data.template_id.clone()),
        template_val: template_data.template_val.clone(),
        template_text: template_data.template_text.clone(),
        created_at: Some(template_data.created_at.clone()),
        created_by: Some(template_data.created_by.clone()),
    };

    let mut datas = Vec::new();

    datas.push(resp_template_data);

    resp_template.datas = Some(datas);

    Ok(resp_template)
}

async fn find_roles_by_app(
    host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    _db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyRole>>> {
    debug!("find_all_roles -> starts");
    match raw_find_roles_by_app(host, db_pool) {
        Ok(res) => {
            debug!("find_roles_by_app -> success to find roles: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_roles_by_app -> failed to find roles, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_roles_by_app(
    host: HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Vec<ReqHtyRole>> {
    let domain = (*host).clone();
    let app = HtyApp::find_by_domain(&domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let roles = app.find_linked_roles(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let mut res = Vec::new();
    for role in roles {
        if role.role_status == APP_STATUS_ACTIVE
            && (role.role_key != "ROOT" && role.role_key != "ADMIN")
        {
            let req_role = ReqHtyRole {
                hty_role_id: Some(role.hty_role_id),
                user_app_info_id: None,
                app_ids: None,
                role_key: Some(role.role_key),
                role_desc: role.role_desc,
                role_status: Some(role.role_status),
                labels: None,
                actions: None,
                style: role.style.clone(),
                role_name: role.role_name.clone(),
            };
            res.push(req_role);
        }
    }
    Ok(res)
}

async fn find_all_roles(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyRole>>> {
    debug!("find_all_roles -> starts");
    match raw_find_all_roles(db_pool) {
        Ok(res) => {
            debug!("find_all_roles -> success to find roles: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_all_roles -> failed to find roles, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_roles(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyRole>> {
    let mut res = Vec::new();
    return match HtyRole::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut()) {
        Ok(roles) => {
            for role in roles {
                let (actions, labels) = role.find_linked_action_and_label(
                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                )?;
                let req_actions: Vec<ReqHtyAction> = actions
                    .iter()
                    .map(|action| ReqHtyAction {
                        hty_action_id: Some(action.clone().hty_action_id),
                        action_name: Some(action.clone().action_name),
                        action_desc: action.clone().action_desc,
                        action_status: Some(action.action_status.clone()),
                        roles: None,
                        labels: None,
                    })
                    .collect();

                let req_labels: Vec<ReqHtyLabel> = labels
                    .iter()
                    .map(|label| ReqHtyLabel {
                        hty_label_id: Some(label.clone().hty_label_id),
                        label_name: Some(label.clone().label_name),
                        label_desc: label.clone().label_desc,
                        label_status: Some(label.clone().label_status),
                        roles: None,
                        actions: None,
                        style: label.style.clone(),
                    })
                    .collect();

                let req_role = ReqHtyRole {
                    hty_role_id: Some(role.clone().hty_role_id),
                    user_app_info_id: None,
                    app_ids: None,
                    role_key: Some(role.clone().role_key),
                    role_desc: role.clone().role_desc,
                    role_status: Some(role.clone().role_status),
                    labels: Some(req_labels),
                    actions: Some(req_actions),
                    style: role.style.clone(),
                    role_name: role.role_name.clone(),
                };
                res.push(req_role);
            }
            Ok(res)
        }
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("find_all_roles -> find all role error ".to_string() + &e.to_string()),
        })),
    };
}

async fn find_all_users(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyUserWithInfos>>> {
    debug!("find_all_users -> starts");
    match raw_find_all_users(db_pool) {
        Ok(res) => {
            debug!("find_all_users -> success to find users: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_all_users -> failed to find users, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_users(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyUserWithInfos>> {
    let users = HtyUser::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let ret: anyhow::Result<Vec<ReqHtyUserWithInfos>> = users
        .iter()
        .map(|user| {
            raw_find_user_with_info_by_id(
                &user.hty_id.clone(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            )
        })
        .collect();
    ret
}

async fn count_unread_tongzhis_by_user_id_and_role_id(
    _sudoer: HtySudoerTokenHeader,
    Query(params): Query<HashMap<String, String>>,
    conn: db::DbConn,
) -> Json<HtyResponse<i32>> {
    let id_user = get_some_from_query_params::<String>("user_id", &params);
    let id_role = get_some_from_query_params::<String>("role_id", &params);

    if id_user.is_none() || id_user.is_none() {
        return wrap_json_hty_err::<i32>(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("id_user or id_user or score_val is none".into()),
        });
    }

    match raw_count_unread_tongzhis_by_user_id_and_role_id(
        &id_user.clone().unwrap(),
        &id_role.clone().unwrap(),
        extract_conn(conn).deref_mut(),
    ) {
        Ok(resp) => {
            debug!(
                "count_unread_tongzhis_by_user_id_and_role_id -> success to find count: {:?}!",
                resp
            );
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!(
                "count_unread_tongzhis_by_user_id_and_role_id -> failed to find users, e: {:?}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

async fn clear_all_unread_tongzhis_by_user_id_and_role_id(
    _sudoer: HtySudoerTokenHeader,
    Query(params): Query<HashMap<String, String>>,
    conn: db::DbConn,
) -> Json<HtyResponse<i32>> {
    debug!(
        "clear_all_unread_tongzhis_by_user_id_and_role_id -> PARAMS: {:?}",
        &params
    );
    let id_role = get_some_from_query_params::<String>("role_id", &params);
    let id_user = get_some_from_query_params::<String>("user_id", &params);

    debug!(
        "clear_all_unread_tongzhis_by_user_id_and_role_id -> id_role: {:?} / id_user: {:?}",
        id_role, id_user
    );

    if id_role.is_none() || id_user.is_none() {
        return wrap_json_hty_err::<i32>(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("user_id or role_id is none".into()),
        });
    }

    match raw_clear_all_unread_tongzhis_by_user_id_and_role_id(
        &id_user.clone().unwrap(),
        &id_role.clone().unwrap(),
        extract_conn(conn).deref_mut(),
    ) {
        Ok(resp) => {
            debug!(
                "clear_all_unread_tongzhis_by_user_id_and_role_id -> OK: {:?}!",
                resp
            );
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!(
                "clear_all_unread_tongzhis_by_user_id_and_role_id -> FAILED, e: {:?}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_clear_all_unread_tongzhis_by_user_id_and_role_id(
    id_user: &String,
    id_role: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<i32> {
    HtyTongzhi::clear_all_unread_tongzhis_by_user_id_and_role_id(id_user, id_role, conn)
}

async fn delete_all_tongzhis_by_status_and_role_id_and_user_id(
    _sudoer: HtySudoerTokenHeader,
    Query(params): Query<HashMap<String, String>>,
    conn: db::DbConn,
) -> Json<HtyResponse<i32>> {
    debug!(
        "delete_all_tongzhis_by_status_and_role_id_and_user_id -> PARAMS: {:?}",
        &params
    );

    let tongzhi_status = get_some_from_query_params::<String>("tongzhi_status", &params);
    let id_role = get_some_from_query_params::<String>("role_id", &params);
    let id_user = get_some_from_query_params::<String>("user_id", &params);

    debug!("delete_all_tongzhis_by_status_and_role_id_and_user_id -> tongzhi_status: {:?} / id_role: {:?} / id_user: {:?}", tongzhi_status, id_role, id_user);

    if tongzhi_status.is_none() || id_role.is_none() || id_user.is_none() {
        return wrap_json_hty_err::<i32>(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("status or user_id or role_id is none".into()),
        });
    }

    match raw_delete_all_tongzhis_by_status_and_role_id_and_user_id(
        &id_role.clone().unwrap(),
        &id_user.clone().unwrap(),
        &tongzhi_status.clone().unwrap(),
        extract_conn(conn).deref_mut(),
    ) {
        Ok(resp) => {
            debug!(
                "delete_all_tongzhis_by_status_and_role_id_and_user_id -> OK: {:?}!",
                resp
            );
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!(
                "delete_all_tongzhis_by_status_and_role_id_and_user_id -> FAILED, e: {:?}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_delete_all_tongzhis_by_status_and_role_id_and_user_id(
    id_role: &String,
    id_user: &String,
    status: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<i32> {
    HtyTongzhi::delete_all_by_status_and_role_id_and_user_id(status, id_role, id_user, conn)
}

fn raw_count_unread_tongzhis_by_user_id_and_role_id(
    id_user: &String,
    id_role: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<i32> {
    HtyTongzhi::count_unread_tongzhis_by_user_id_and_role_id(id_user, id_role, conn)
}

// find_users by keyword
async fn find_users(
    Query(params): Query<HashMap<String, String>>,
    _sudoer: HtySudoerTokenHeader,
    _conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<(Vec<ReqHtyUserWithInfos>, i64, i64)>> {
    debug!("find_users -> starts");
    let (page, page_size) = get_page_and_page_size(&params);

    debug!("find_users -> {:?}", &params);

    let keyword = get_some_from_query_params::<String>("keyword", &params);

    match raw_find_users(&page, &page_size, &keyword, db_pool) {
        Ok(resp) => {
            debug!("find_users -> success to find users: {:?}!", resp);
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!("find_users -> failed to find users, e: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_users(
    page: &Option<i64>,
    page_size: &Option<i64>,
    keyword: &Option<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<(Vec<ReqHtyUserWithInfos>, i64, i64)> {
    let (found_users, total_page, total) = HtyUser::find_users_by_keyword(
        page,
        page_size,
        keyword,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    let mut resp_users_with_info = Vec::new();

    for the_user in found_users {
        let user_with_info = raw_find_user_with_info_by_id(
            &the_user.hty_id.clone(),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        resp_users_with_info.push(user_with_info);
    }

    Ok((resp_users_with_info, total_page, total))
}


// find_hty_users by keyword with out user_info data
async fn find_hty_users_by_keyword(
    Query(params): Query<HashMap<String, String>>,
    _sudoer: HtySudoerTokenHeader,
    _conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<(Vec<ReqHtyUser>, i64, i64)>> {
    debug!("find_hty_users_by_keyword -> starts");
    let (page, page_size) = get_page_and_page_size(&params);

    debug!("find_hty_users_by_keyword -> {:?}", &params);

    let keyword = get_some_from_query_params::<String>("keyword", &params);

    match raw_find_hty_users_by_keyword(&page, &page_size, &keyword, db_pool) {
        Ok(resp) => {
            debug!("raw_find_hty_users_by_keyword -> success to find users: {:?}!", resp);
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!("raw_find_hty_users_by_keyword -> failed to find users, e: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_hty_users_by_keyword(
    page: &Option<i64>,
    page_size: &Option<i64>,
    keyword: &Option<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<(Vec<ReqHtyUser>, i64, i64)> {
    let (found_users, total_page, total) = HtyUser::find_users_by_keyword(
        page,
        page_size,
        keyword,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    let req_users = HtyUser::to_req_users(&found_users);
    Ok((req_users, total_page, total))
}

async fn find_all_apps_with_roles(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyApp>>> {
    debug!("find_all_apps_with_roles -> starts");
    match raw_find_all_apps_with_roles(db_pool) {
        Ok(res) => {
            debug!(
                "find_all_apps_with_roles -> success to find apps: {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "find_all_apps_with_roles -> failed to find apps with roles, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_apps_with_roles(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyApp>> {
    let apps = HtyApp::find_all_apps(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let res: Option<Vec<ReqHtyApp>> = apps
        .iter()
        .map(|app| {
            let req_app = raw_find_app_with_roles(&app.app_id.clone(), &db_pool).ok();
            req_app
        })
        .collect();
    res.ok_or(anyhow!(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some("get_all_apps_with_roles -> raw_get_all_apps_with_roles ".to_string()),
    }))
}

async fn find_role_by_key(
    Path(key): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyRole>> {
    debug!("find_role_by_key -> starts");
    match raw_find_role_by_key(&key, extract_conn(conn).deref_mut()) {
        Ok(res) => {
            debug!("find_role_by_key -> success to find app: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_role_by_key -> failed to find app, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_role_by_key(key: &String, conn: &mut PgConnection) -> anyhow::Result<ReqHtyRole> {
    Ok(HtyRole::find_by_key(key, conn)?.to_req())
}

async fn find_app_by_domain(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<ReqHtyApp>> {
    debug!("find_app_by_domain -> starts, host: {:?}", host);
    match raw_find_app_by_domain(&host, db_pool) {
        Ok(res) => {
            debug!("find_app_by_domain -> success to find app: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_app_by_domain -> failed to find app, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_app_by_domain(
    host: &HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyApp> {
    let domain = (*host).clone();
    let app = HtyApp::find_by_domain(&domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let gonggaos = HtyGongGao::find_by_app_id(
        &app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let res = ReqHtyApp {
        app_id: Some(app.app_id),
        wx_id: app.wx_id,
        wx_secret: app.wx_secret,
        domain: app.domain,
        app_desc: app.app_desc,
        app_status: Some(app.app_status),
        role_ids: None,
        roles: None,
        gonggaos: Some(gonggaos),
        tags: None,
        pubkey: app.pubkey,
        privkey: app.privkey,
        needs_refresh: None,
        is_wx_app: app.is_wx_app.clone(),
    };
    Ok(res)
}

// todo: @buddy 返回使用`Vec<ReqTagRefsByRefId>`
async fn find_tags_by_ref_ids(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    ref_ids: Json<Vec<String>>,
) -> Json<HtyResponse<Vec<ReqTagRefsByRefId>>> {
    debug!("find_tags_by_ref_ids -> starts");
    let ids = ref_ids.0;

    match raw_find_tags_by_ref_ids(ids, db_pool) {
        Ok(res) => {
            debug!("find_tags_by_ref_ids -> success to find tags: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_tags_by_ref_ids -> failed to find tags, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_tags_by_ref_ids(
    ref_ids: Vec<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Vec<ReqTagRefsByRefId>> {
    let mut res = vec![];
    for ref_id in ref_ids {
        let req_tags = raw_find_tags_by_ref_id(ref_id, db_pool.clone())?;
        res.push(req_tags);
    }
    Ok(res)
}

async fn bulk_update_tag_ref(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_tag_refs): Json<ReqTagRefsByRefId>,
) -> Json<HtyResponse<ReqTagRefsByRefId>> {
    // 这一组`tag_ref`都是一个`ref_id`的。
    // 所以删掉这一组`ref_id`下所有的`tag_ref`数据。
    // 然后再添加所有的`tag_refs`.
    // 返回是创建好的新的一组`TagRefs`(带生成后的主键id)
    debug!("bulk_update_tag_ref -> starts");
    match raw_bulk_update_tag_ref(req_tag_refs, db_pool) {
        Ok(res) => {
            debug!("bulk_update_tag_ref -> success to update: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("bulk_update_tag_ref -> failed to update, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_bulk_update_tag_ref(
    req_tag_refs_by_ref_id: ReqTagRefsByRefId,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqTagRefsByRefId> {
    let ref_id = req_tag_refs_by_ref_id.ref_id.clone().unwrap();

    let cloned_req_tag_refs = req_tag_refs_by_ref_id.tag_refs.clone().unwrap_or(Vec::new());

    let mut cloned_req_tag_refs_by_ref_id = req_tag_refs_by_ref_id.clone();

    let task_update = move |_in_params: Option<HashMap<String, String>>,
                            conn: &mut PgConnection|
                            -> anyhow::Result<Vec<ReqHtyTagRef>> {
        let _ = HtyTagRef::delete_all_by_ref_id(&ref_id, conn)?;

        let mut res_tag_refs = vec![];
        for req_tag_ref in cloned_req_tag_refs.clone() {
            if req_tag_ref.hty_tag_id.is_none() || req_tag_ref.ref_type.is_none() {
                return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("hty_tag_id or ref_type can not be none".into())
                }));
            }
            let mut new_req_tag_ref = req_tag_ref.clone();
            let new_id = uuid();
            let db_tag_ref = HtyTagRef {
                the_id: new_id.clone(),
                hty_tag_id: req_tag_ref.hty_tag_id.clone().unwrap(),
                ref_id: ref_id.clone(),
                ref_type: req_tag_ref.ref_type.clone().unwrap(),
                meta: req_tag_ref.meta.clone(),
            };
            let _ = HtyTagRef::create(&db_tag_ref, conn)?;
            new_req_tag_ref.tag_ref_id = Some(new_id.clone());
            res_tag_refs.push(new_req_tag_ref.clone());
        }
        Ok(res_tag_refs)
    };
    let params = HashMap::new();
    let res_tag_refs = exec_read_write_task(
        Box::new(task_update),
        Some(params),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    cloned_req_tag_refs_by_ref_id.tag_refs = Some(res_tag_refs.clone());
    Ok(cloned_req_tag_refs_by_ref_id)
}

async fn find_tags_by_ref_id(
    Path(ref_id): Path<String>,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<ReqTagRefsByRefId>> {
    debug!("find_tags_by_ref_id -> starts");
    match raw_find_tags_by_ref_id(ref_id, db_pool) {
        Ok(res) => {
            debug!("find_tags_by_ref_id -> success to find tags: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_tags_by_ref_id -> failed to find tags, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_tags_by_ref_id(
    id_ref: String,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqTagRefsByRefId> {
    let mut res = ReqTagRefsByRefId {
        ref_id: Some(id_ref.clone()),
        ref_type: None,
        tag_refs: None,
    };
    let tag_refs =
        HtyTagRef::find_all_by_ref_id(&id_ref, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let mut req_tag_refs = vec![];
    for tag_ref in tag_refs {
        let db_tag = HtyTag::find_by_tag_id(
            &tag_ref.hty_tag_id,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        let req_tag_ref = ReqHtyTagRef {
            tag_ref_id: Some(tag_ref.the_id.clone()),
            hty_tag_id: Some(db_tag.tag_id.clone()),
            ref_id: Some(tag_ref.ref_id.clone()),
            ref_type: Some(tag_ref.ref_type.clone()),
            meta: tag_ref.meta.clone(),
            tag: Some(db_tag.to_req()),
        };
        res.ref_type = Some(tag_ref.ref_type.clone());
        req_tag_refs.push(req_tag_ref);
    }
    res.tag_refs = Some(req_tag_refs);
    Ok(res)
}

async fn find_app_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(app_id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<ReqHtyApp>> {
    debug!("find_app_by_id -> starts");
    match raw_find_app_by_id(&app_id, db_pool) {
        Ok(res) => {
            debug!("find_app_by_id -> success to find app: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_app_by_id -> failed to find app, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_app_by_id(id_app: &String, db_pool: Arc<DbState>) -> anyhow::Result<ReqHtyApp> {
    let app = HtyApp::find_by_id(id_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let gonggaos = HtyGongGao::find_by_app_id(
        &app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let res = ReqHtyApp {
        app_id: Some(app.app_id),
        wx_id: app.wx_id,
        wx_secret: app.wx_secret,
        domain: app.domain,
        app_desc: app.app_desc,
        app_status: Some(app.app_status),
        role_ids: None,
        roles: None,
        gonggaos: Some(gonggaos),
        tags: None,
        pubkey: app.pubkey,
        privkey: app.privkey,
        needs_refresh: None,
        is_wx_app: app.is_wx_app.clone(),
    };
    Ok(res)
}

async fn find_all_tongzhis_with_page(
    Query(params): Query<HashMap<String, String>>,
    _auth: AuthorizationHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<(Vec<HtyTongzhi>, i64, i64)>> {
    debug!("find_all_tongzhis_with_page -> starts");
    let (page, page_size) = get_page_and_page_size(&params);

    let mut the_tongzhi_status = None;
    let mut the_role_id = None;
    let mut the_hty_id = None;

    if params.get("tongzhi_status").is_some() {
        // READ / UNREAD
        the_tongzhi_status = Some(
            params
                .get("tongzhi_status")
                .unwrap()
                .parse::<String>()
                .unwrap_or_default(),
        );
    }

    if params.get("role_id").is_some() {
        the_role_id = Some(
            params
                .get("role_id")
                .unwrap()
                .parse::<String>()
                .unwrap_or_default(),
        );
    }

    if params.get("hty_id").is_some() {
        the_hty_id = Some(
            params
                .get("hty_id")
                .unwrap()
                .parse::<String>()
                .unwrap_or_default(),
        );
    }

    match raw_find_tongzhis(
        &page,
        &page_size,
        &the_tongzhi_status,
        &the_role_id,
        &the_hty_id,
        db_pool,
    ) {
        Ok(res) => {
            debug!("raw_find_tongzhis -> success to find actions: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("raw_find_tongzhis -> failed to find actions, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn find_tongzhis(
    Query(params): Query<HashMap<String, String>>,
    auth: AuthorizationHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<(Vec<HtyTongzhi>, i64, i64)>> {
    debug!("find_tongzhis -> starts");
    let (page, page_size) = get_page_and_page_size(&params);

    let mut the_tongzhi_status = None;
    let mut the_role_id = None;
    let the_hty_id;

    match jwt_decode_token(&(*auth).clone()) {
        Ok(decode_res) => {
            the_hty_id = decode_res.hty_id;
        }
        Err(e) => {
            return wrap_json_anyhow_err(anyhow!(e))
        }
    }

    if params.get("tongzhi_status").is_some() {
        // READ / UNREAD
        the_tongzhi_status = Some(
            params
                .get("tongzhi_status")
                .unwrap()
                .parse::<String>()
                .unwrap_or_default(),
        );
    }

    if params.get("role_id").is_some() {
        the_role_id = Some(
            params
                .get("role_id")
                .unwrap()
                .parse::<String>()
                .unwrap_or_default(),
        );
    }

    match raw_find_tongzhis(
        &page,
        &page_size,
        &the_tongzhi_status,
        &the_role_id,
        &the_hty_id,
        db_pool,
    ) {
        Ok(res) => {
            debug!("find_tongzhis -> success to find actions: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_tongzhis -> failed to find actions, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_tongzhis(
    page: &Option<i64>,
    page_size: &Option<i64>,
    the_tongzhi_status: &Option<String>,
    the_role_id: &Option<String>,
    the_hty_id: &Option<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<(Vec<HtyTongzhi>, i64, i64)> {
    let res = HtyTongzhi::find_tongzhis_with_page(
        page,
        page_size,
        the_tongzhi_status,
        the_role_id,
        &the_hty_id.as_ref(),
        &None,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(res)
}

async fn delete_tag_ref(
    _sudoer: HtySudoerTokenHeader,
    Path(tag_ref_id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<()>> {
    debug!("delete_tag_ref -> starts");
    match raw_delete_tag_ref(tag_ref_id, db_pool) {
        Ok(res) => {
            debug!("delete_tag_ref -> success to delete tag_ref!");
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("delete_tag_ref -> failed to delete tag_ref, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_delete_tag_ref(id_to_delete: String, db_pool: Arc<DbState>) -> anyhow::Result<()> {
    let _ = HtyTagRef::delete_by_id(
        &id_to_delete,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}


async fn create_or_update_tags(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_tag): Json<ReqHtyTag>,
) -> Json<HtyResponse<()>> {
    debug!("create_or_update_tags -> starts");
    match raw_create_or_update_tags(req_tag, db_pool) {
        Ok(res) => {
            debug!(
                "raw_create_or_update_tags -> success to create or update tag: {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "raw_create_or_update_tags -> failed to create or update tag, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_tags(req_tag: ReqHtyTag, db_pool: Arc<DbState>) -> anyhow::Result<()> {
    if !req_tag.tag_id.is_none() {
        let id_tag = req_tag.tag_id.clone().unwrap();
        let mut db_tag =
            HtyTag::find_by_tag_id(&id_tag, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
        if req_tag.tag_name.is_some() {
            db_tag.tag_name = req_tag.tag_name.clone().unwrap();
        }
        db_tag.tag_desc = req_tag.tag_desc.clone();
        db_tag.style = req_tag.style.clone();
        let _ = HtyTag::update(&db_tag, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
        return Ok(());
    }
    if req_tag.tag_name.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("for create flow, tag_name can not be none".into())
        }));
    }
    let db_tag = HtyTag {
        tag_id: uuid(),
        tag_name: req_tag.tag_name.clone().unwrap(),
        tag_desc: req_tag.tag_desc.clone(),
        style: req_tag.style.clone(),
    };
    let _ = HtyTag::create(&db_tag, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    Ok(())
}

async fn create_tag_ref(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_tag_ref): Json<ReqHtyTagRef>,
) -> Json<HtyResponse<String>> {
    debug!("create_tag_ref -> starts");
    match raw_create_tag_ref(req_tag_ref, db_pool) {
        Ok(res) => {
            debug!("create_tag_ref -> success to create tag_ref: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("create_tag_ref -> failed to create tag_ref, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_tag_ref(req_tag_ref: ReqHtyTagRef, db_pool: Arc<DbState>) -> anyhow::Result<String> {
    if req_tag_ref.hty_tag_id.is_none()
        || req_tag_ref.ref_id.is_none()
        || req_tag_ref.ref_type.is_none()
    {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("hty_tag_id or ref_id or ref_type is none".into())
        }));
    }
    let tag_id = req_tag_ref.hty_tag_id.clone().unwrap();
    let ref_id = req_tag_ref.ref_id.clone().unwrap();
    let duplicate_check = HtyTagRef::verify_exist_by_ref_id_and_tag_id(
        &tag_id,
        &ref_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    if duplicate_check {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("duplicate record for the tag_id and ref_id".into())
        }));
    }
    let db_tag_ref = HtyTagRef {
        the_id: uuid(),
        hty_tag_id: req_tag_ref.hty_tag_id.clone().unwrap(),
        ref_id: req_tag_ref.ref_id.clone().unwrap(),
        ref_type: req_tag_ref.ref_type.clone().unwrap(),
        meta: req_tag_ref.meta.clone(),
    };
    let res = HtyTagRef::create(
        &db_tag_ref,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(res.the_id)
}

async fn find_all_tags(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyTag>>> {
    debug!("find_all_tags -> starts");
    match raw_find_all_tags(db_pool) {
        Ok(res) => {
            debug!("find_all_tags -> success to find tags: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_all_tags -> failed to find tags, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_tags(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyTag>> {
    let mut res = Vec::new();
    let tags = HtyTag::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let req_tags: Vec<ReqHtyTag> = tags.into_iter().map(|tag| tag.to_req()).collect();
    for mut req_tag in req_tags {
        let id_tag = req_tag.tag_id.clone().unwrap();
        let tag_refs = HtyTagRef::find_all_by_tag_id(
            &id_tag,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        let req_tag_refs = HtyTagRef::all_to_reqs(&tag_refs);
        req_tag.refs = Some(req_tag_refs);
        res.push(req_tag.clone())
    }
    Ok(res)
}

async fn find_all_actions(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyAction>>> {
    debug!("find_all_actions -> starts");
    match raw_find_all_actions(db_pool) {
        Ok(res) => {
            debug!("find_all_actions -> success to find actions: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_all_actions -> failed to find actions, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_actions(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyAction>> {
    let mut res = Vec::new();
    return match HtyAction::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut()) {
        Ok(actions) => {
            for action in actions {
                let (roles, labels) = action.find_linked_role_and_label(
                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                )?;
                let req_roles: Vec<ReqHtyRole> = roles
                    .iter()
                    .map(|role| ReqHtyRole {
                        hty_role_id: Some(role.clone().hty_role_id),
                        user_app_info_id: None,
                        app_ids: None,
                        role_key: Some(role.clone().role_key),
                        role_desc: role.clone().role_desc,
                        role_status: Some(role.clone().role_status),
                        labels: None,
                        actions: None,
                        style: role.style.clone(),
                        role_name: role.role_name.clone(),
                    })
                    .collect();
                let req_labels: Vec<ReqHtyLabel> = labels
                    .iter()
                    .map(|label| ReqHtyLabel {
                        hty_label_id: Some(label.clone().hty_label_id),
                        label_name: Some(label.clone().label_name),
                        label_desc: label.clone().label_desc,
                        label_status: Some(label.clone().label_status),
                        roles: None,
                        actions: None,
                        style: label.style.clone(),
                    })
                    .collect();
                let req_action = ReqHtyAction {
                    hty_action_id: Some(action.clone().hty_action_id),
                    action_name: Some(action.clone().action_name),
                    action_desc: action.clone().action_desc,
                    action_status: Some(action.clone().action_status),
                    roles: Some(req_roles),
                    labels: Some(req_labels),
                };
                res.push(req_action);
            }
            Ok(res)
        }
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("find_all_actions -> find all action error ".to_string() + &e.to_string()),
        })),
    };
}

async fn find_all_labels(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyLabel>>> {
    debug!("find_all_labels -> starts");
    match raw_find_all_labels(db_pool) {
        Ok(res) => {
            debug!("find_all_labels -> success to find labels: {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("find_all_labels -> failed to find labels, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_labels(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyLabel>> {
    let mut res = Vec::new();
    return match HtyLabel::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut()) {
        Ok(labels) => {
            for label in labels {
                let (roles, actions) = label.find_linked_role_and_action(
                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                )?;
                let req_roles: Vec<ReqHtyRole> = roles
                    .iter()
                    .map(|role| ReqHtyRole {
                        hty_role_id: Some(role.clone().hty_role_id),
                        user_app_info_id: None,
                        app_ids: None,
                        role_key: Some(role.clone().role_key),
                        role_desc: role.clone().role_desc,
                        role_status: Some(role.role_status.clone()),
                        labels: None,
                        actions: None,
                        style: role.style.clone(),
                        role_name: role.role_name.clone(),
                    })
                    .collect();
                let req_actions: Vec<ReqHtyAction> = actions
                    .iter()
                    .map(|action| ReqHtyAction {
                        hty_action_id: Some(action.clone().hty_action_id),
                        action_name: Some(action.clone().action_name),
                        action_desc: action.clone().action_desc,
                        action_status: Some(action.clone().action_status),
                        roles: None,
                        labels: None,
                    })
                    .collect();
                let req_label = ReqHtyLabel {
                    hty_label_id: Some(label.clone().hty_label_id),
                    label_name: Some(label.clone().label_name),
                    label_desc: label.clone().label_desc,
                    label_status: Some(label.clone().label_status),
                    roles: Some(req_roles),
                    actions: Some(req_actions),
                    style: label.style.clone(),
                };
                res.push(req_label);
            }
            Ok(res)
        }
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("find_all_labels -> find all label error ".to_string() + &e.to_string()),
        })),
    };
}

async fn register_verify(
    host: HtyHostHeader,
    conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(register_verify): Json<ReqRegisterVerify>,
) -> Json<HtyResponse<String>> {
    debug!("register_verify -> starts: register_verify: {:?}", register_verify);
    match raw_register_verify(register_verify, host, conn, db_pool).await {
        Ok(res) => {
            debug!("register_verify -> success to verify!");
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("register_verify -> failed to verify, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_register_verify(
    register_verify: ReqRegisterVerify,
    _host: HtyHostHeader,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    debug!(
        "raw_register_verify -> register_verify: {:?}",
        &register_verify
    );

    let register_verify_copy = register_verify.clone();

    let in_app = HtyApp::find_by_id(&register_verify.clone().app_id.unwrap(),
                                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    debug!("raw_register_verify -> in_app: {:?}", in_app);
    let user_id = register_verify_copy.hty_id.unwrap();
    let verified = register_verify_copy.validate.unwrap();

    let mut user_info = UserAppInfo::find_by_hty_id_and_app_id(
        &user_id,
        &in_app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    user_info.is_registered = verified;

    debug!("raw_register_verify -> user_info: {:?}", &user_info);
    if verified {
        let task_create = move |_in_params: Option<HashMap<String, ReqHtyRole>>,
                                conn: &mut PgConnection|
                                -> anyhow::Result<()> {
            let _ = UserAppInfo::update(&user_info, conn)?;
            Ok(())
        };
        let params = HashMap::new();
        let _ = exec_read_write_task(
            Box::new(task_create),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
    } else {
        if register_verify_copy.reject_reason.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("Reject reason can not be NULL when validate is false".into()),
            }));
        }
        user_info.reject_reason = register_verify_copy.reject_reason;
        let _ = UserAppInfo::update(
            &user_info,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
    }
    Ok(serde_json::to_string(&register_verify.clone())?)
}

async fn register(
    host: HtyHostHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(register_info): Json<ReqRegister>,
) -> Json<HtyResponse<ReqHtyUserWithInfos>> {
    debug!("register -> starts");

    match raw_register(register_info, host, db_conn, db_pool).await {
        Ok(res) => {
            debug!("register -> success to register : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("register -> failed to register, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_register(
    register_info: ReqRegister,
    host: HtyHostHeader,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyUserWithInfos> {
    debug!("raw_register -> register_info -> {:?}", &register_info);

    let in_unionid = register_info.clone().unionid;

    debug!(
        "raw_register -> in_unionid -> {:?} / is_none? {:?} / is_empty? {:?}",
        &in_unionid,
        &in_unionid.is_none(),
        &in_unionid.clone().unwrap().trim().is_empty()
    );
    if in_unionid.is_none() || in_unionid.clone().unwrap().trim().is_empty() {
        let msg = format!("注册信息里面没有用户小程序UNIONID / {:?}", &register_info);
        debug!("raw_register -> NO UNIONID -> {:?}", msg);
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(msg),
        }));
    }

    let in_openid = register_info.clone().openid;

    debug!("raw_register -> in_openid -> {:?}", &in_openid,);
    if in_openid.is_none() || in_openid.unwrap().trim().is_empty() {
        let msg = format!("注册信息里面没有用户小程序OPENID / {:?}", &register_info);
        debug!("raw_register -> NO OPENID -> {:?}", msg);
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(msg),
        }));
    }

    match HtyUser::find_by_union_id(
        &in_unionid.clone().unwrap(),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    ) {
        Ok(user) => {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some(format!("用户已经存在！USER -> {:?}", &user)),
            }));
        }
        Err(_) => {
            // ok
        }
    }

    let in_app = get_app_from_host(
        (*host).clone(),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    let to_create_user = HtyUser {
        hty_id: uuid(),
        union_id: register_info.unionid.clone(),
        enabled: register_info.enabled.unwrap_or(false),
        created_at: Some(current_local_datetime()),
        real_name: register_info.real_name.clone(),
        sex: register_info.sex.clone(),
        mobile: register_info.mobile.clone(),
        settings: register_info.user_settings.clone(),
    };

    debug!("raw_register -> to_create_user -> {:?}", to_create_user);

    let to_create_user_hty_id = to_create_user.hty_id.clone();
    let _to_create_user_union_id = to_create_user.union_id.clone();


    let mut to_create_user_from_app_info = UserAppInfo {
        hty_id: to_create_user.hty_id.clone(),
        app_id: Some(in_app.app_id.clone()),
        openid: register_info.openid.clone(),
        is_registered: false,
        id: uuid(),
        username: None,
        password: None,
        meta: register_info.meta.clone(),
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    debug!("raw_register -> to_create_user_from_app_info -> {:?}", to_create_user_from_app_info);

    let to_create_user_in_app_info_id = to_create_user_from_app_info.id.clone();

    let mut to_create_from_user_info_role = UserInfoRole {
        the_id: uuid(),
        user_info_id: to_create_user_in_app_info_id.clone(),
        role_id: "".to_string(),
    };

    match register_info.clone().role {
        Some(role) => match role.as_str() {
            "TEACHER" => {
                to_create_user_from_app_info.teacher_info = register_info.teacher_info.clone();
                let role = HtyRole::find_by_key(
                    "TEACHER",
                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                )?;
                to_create_from_user_info_role.role_id = role.hty_role_id;
            }
            "STUDENT" => {
                to_create_user_from_app_info.student_info = register_info.student_info.clone();
                let role = HtyRole::find_by_key(
                    "STUDENT",
                    extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                )?;
                to_create_from_user_info_role.role_id = role.hty_role_id;
            }
            _ => {
                return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("Role must be teacher or student".into()),
                }));
            }
        },
        _ => {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("invalid role".into()),
            }));
        }
    }

    // ------from app user info created only once
    let _created_user = HtyUser::create(
        &to_create_user,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let _created_user_from_app_info = UserAppInfo::create(
        &to_create_user_from_app_info,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let _created_user_info_role = UserInfoRole::create(
        &to_create_from_user_info_role,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    // setup response value here(we don't return to_app infos, so we set response here.)
    let mut resp_user_with_infos = ReqHtyUserWithInfos {
        hty_id: Some(to_create_user.hty_id.clone()),
        union_id: to_create_user.union_id.clone(),
        enabled: Some(to_create_user.enabled),
        created_at: to_create_user.created_at.clone(),
        real_name: to_create_user.real_name.clone(),
        sex: to_create_user.sex.clone(),
        mobile: to_create_user.mobile.clone(),
        infos: None,
        info_roles: None,
        settings: to_create_user.settings.clone(),
    };

    debug!("raw_register -> resp_user_with_infos -> {:?}", resp_user_with_infos);


    // 只返回`from_app`信息
    let mut resp_user_infos = Vec::new();
    let resp_from_app_user_info = to_create_user_from_app_info.to_req();

    resp_user_infos.push(resp_from_app_user_info);

    let resp_role = ReqUserInfoRole {
        the_id: Some(to_create_from_user_info_role.the_id.clone()),
        user_info_id: None,
        role_id: None,
    };

    let mut resp_roles = Vec::new();

    resp_roles.push(resp_role);

    resp_user_with_infos.infos = Some(resp_user_infos);
    resp_user_with_infos.info_roles = Some(resp_roles);

    // ----- create to_app infos ----------
    let to_apps = AppFromTo::find_all_active_to_apps_by_from_app(
        &in_app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    for to_app in to_apps {
        let to_app_id = to_app.clone().to_app_id;

        debug!("raw_register() -> to_app_id {:?}", &to_app_id);

        let to_user_app_info = UserAppInfo {
            hty_id: to_create_user_hty_id.clone(),
            app_id: Some(to_app_id.clone()),
            openid: None,
            is_registered: false,
            id: uuid(),
            username: None,
            password: None,
            meta: None,
            created_at: Some(current_local_datetime()),
            teacher_info: None,
            student_info: None,
            reject_reason: None,
            needs_refresh: Some(false),
            avatar_url: None,
        };

        let params = HashMap::new();

        let to_user_app_info_copy = to_user_app_info.clone();
        let to_create_user_from_app_info_copy = to_create_user_from_app_info.clone();

        let create_user_info_task = move |_in_params: Option<HashMap<String, ReqHtyRole>>,
                                          conn: &mut PgConnection|
                                          -> anyhow::Result<ReqUserAppInfo> {
            let _ = UserAppInfo::create(&to_user_app_info_copy, conn)?;

            let out_info = ReqUserAppInfo {
                id: Some(to_create_user_from_app_info_copy.id.clone()),
                app_id: None,
                hty_id: None,
                openid: None,
                is_registered: to_create_user_from_app_info_copy.is_registered,
                username: None,
                password: None,
                roles: None,
                meta: None,
                created_at: None,
                teacher_info: None,
                student_info: None,
                reject_reason: None,
                needs_refresh: None,
                unread_tongzhi_count: None,
                avatar_url: None,
            };


            Ok(out_info)

        };

        let _created_user_info = exec_read_write_task(
            Box::new(create_user_info_task),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;

        debug!("raw_register -> start post_registration");

        do_post_registration(&to_app_id.clone(), &to_create_user_hty_id.clone());
    }

    Ok(resp_user_with_infos)
}

fn do_post_registration(to_app_id: &String, user_id: &String) {
    if skip_post_registration() {
        debug!("do_post_registration ::BY_PASSED::");
    } else {
        let to_app_id_copy = to_app_id.clone();
        let user_id_copy = user_id.clone();
        debug!("start raw_register -> update_official_account_openid");

        task::spawn(async move {
            // 刷新用户公众号(`to_app`)的`openid`，这里和`login2_with_unionid`里面的`post_login()`共用方法
            // `refresh_cache_and_get_wx_all_follower_openids()`
            debug!("entering raw_register -> update_official_account_openid");
            let client = reqwest::Client::new();

            let req_struct = ReqUserIdWithAppId {
                app_id: Some(to_app_id_copy),
                hty_id: Some(user_id_copy),
            };

            debug!(
                "raw_register -> update_official_account_openid -> req_struct: {:?}",
                &req_struct
            );

            let body = serde_json::to_string::<ReqUserIdWithAppId>(&req_struct).unwrap();

            debug!(
                "raw_register -> update_official_account_openid -> body: {:?}",
                &body
            );

            let url = format!("{}/update_official_account_openid", get_uc_url(), );

            debug!(
                "raw_register -> update_official_account_openid -> uc_url: {:?}",
                &url
            );

            let resp = client
                .post(url)
                .body(body)
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .send()
                .await;

            debug!(
                "raw_register -> update_official_account_openid -> resp: {:?}",
                &resp
            );
        });
    }
}

async fn create_hty_gonggao(
    _root: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_gonggao): Json<ReqHtyGongGao>,
) -> Json<HtyResponse<()>> {
    debug!("create_hty_gonggao -> starts");
    debug!("create_hty_gonggao -> req_body {:?}", &req_gonggao);
    match raw_create_hty_gonggao(&req_gonggao, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("create_hty_gonggao -> failed to create gonggao, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_create_hty_gonggao(
    req_gonggao: &ReqHtyGongGao,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    let in_gonggap = HtyGongGao {
        id: uuid(),
        app_id: req_gonggao.app_id.clone(),
        created_at: current_local_datetime(),
        gonggao_status: req_gonggao.gonggao_status.clone(),
        content: req_gonggao.content.clone(),
    };

    let _ = HtyGongGao::create(
        &in_gonggap,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}

async fn update_hty_gonggao(
    _root: HtySudoerTokenHeader,
    _db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(req_gonggao): Json<ReqHtyGongGao>,
) -> Json<HtyResponse<()>> {
    debug!("update_hty_gonggao -> starts");
    debug!("update_hty_gonggao -> req_body {:?}", &req_gonggao);
    match raw_update_hty_gonggao(&req_gonggao, _db_conn, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("update_hty_gonggao -> failed to update gonggao, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_update_hty_gonggao(
    req_gonggao: &ReqHtyGongGao,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    debug!("start raw_update_tongzhi_status_by_id");
    if req_gonggao.id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("id is none".into())
        }));
    }
    let gonggao_id = req_gonggao.id.clone().unwrap();
    let mut gonggao = HtyGongGao::find_by_id(
        &gonggao_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    gonggao.gonggao_status = req_gonggao.gonggao_status.clone();
    gonggao.content = req_gonggao.content.clone();
    gonggao.app_id = req_gonggao.app_id.clone();

    let _ = HtyGongGao::update(&gonggao, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    Ok(())
}

async fn update_tongzhi_status_by_id(
    _root: HtySudoerTokenHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    req_body: Json<ReqUpdateTongzhiStatus>,
) -> Json<HtyResponse<()>> {
    debug!("update_tongzhi_status_by_id -> starts");
    debug!("update_tongzhi_status_by_id -> req_body {:?}", &req_body);

    let the_tongzhi_id = req_body.0.tongzhi_id.unwrap();
    let the_tongzhi_status = req_body.0.tongzhi_status.unwrap();

    match raw_update_tongzhi_status_by_id(&the_tongzhi_id, &the_tongzhi_status, db_conn, db_pool)
        .await
    {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!(
                "update_tongzhi_status_by_id -> failed to update status, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_update_tongzhi_status_by_id(
    the_tongzhi_id: &String,
    the_tongzhi_status: &String,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    debug!("start raw_update_tongzhi_status_by_id");

    let mut tongzhi = HtyTongzhi::find_by_id(
        &the_tongzhi_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    tongzhi.tongzhi_status = the_tongzhi_status.clone();

    let _ = HtyTongzhi::update(&tongzhi, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    Ok(())
}

async fn update_official_account_openid(
    _db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(req_user): Json<ReqUserIdWithAppId>,
) -> Json<HtyResponse<()>> {
    debug!("update_official_account_openid -> starts");
    debug!("update_official_account_openid -> req_user {:?}", &req_user);
    debug!("update_official_account_openid -> entering raw_update_official_account_openid");

    match raw_update_official_account_openid(&req_user, _db_conn, db_pool).await {
        Ok(res) => {
            debug!("update_official_account_openid -> success to update!");
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "update_official_account_openid -> failed to update, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_update_official_account_openid(
    req_user: &ReqUserIdWithAppId,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    debug!("start raw_update_official_account_openid");
    let id_user = req_user.hty_id.clone().unwrap();
    let id_app = req_user.app_id.clone().unwrap();

    let to_app = HtyApp::find_by_id(&id_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let in_user =
        HtyUser::find_by_hty_id(&id_user, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let id_union = in_user.union_id.clone().unwrap();

    let _ = refresh_cache_and_get_wx_all_follower_openids(&to_app).await?;

    let mut to_update_info = UserAppInfo::find_by_hty_id_and_app_id(
        &id_user,
        &id_app,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    debug!(
        "raw_update_official_account_openid -> to_update_info: {:?}",
        &to_update_info
    );

    let resp_openid = find_wx_openid_by_unionid_and_hty_app(&id_union, &to_app).await;

    match resp_openid {
        Ok(to_app_open_id) => {
            to_update_info.openid = Some(to_app_open_id);
            to_update_info.is_registered = true;
            to_update_info.reject_reason = None;

            debug!(
                "raw_update_official_account_openid -> to_update_info with openid: {:?}",
                &to_update_info
            );

            let created_info = UserAppInfo::update(
                &to_update_info,
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            )?;

            debug!(
                "raw_update_official_account_openid -> created_info {:?}",
                &created_info
            );
        }
        Err(_) => {
            debug!("raw_update_official_account_openid -> UPDATE user_info / app_open_id 获取失败 / {:?}", &to_update_info);
        }
    }

    Ok(())
}

async fn register_rollback(
    host: HtyHostHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(rollback_info): Json<ReqHtyUserWithInfos>,
) -> Json<HtyResponse<()>> {
    debug!("register_rollback -> starts");
    match raw_register_rollback(rollback_info, host, db_conn, db_pool).await {
        Ok(res) => {
            debug!("register_rollback -> success to rollback!");
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("register_rollback -> failed to rollback, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_register_rollback(
    rollback_info: ReqHtyUserWithInfos,
    _host: HtyHostHeader,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    let hty_id = rollback_info.hty_id.unwrap();
    let infos = rollback_info.infos.clone();

    let task_delete = move |_in_params: Option<HashMap<String, ReqHtyRole>>,
                            conn: &mut PgConnection|
                            -> anyhow::Result<()> {
        // let _ = UserInfoRole::delete_by_id(&id3, extract_conn(conn).deref_mut());
        // let _ = UserAppInfo::delete(&id2, extract_conn(conn).deref_mut());
        let _ = HtyUser::delete_by_hty_id(&hty_id, conn)?;
        for info in infos.clone().unwrap() {
            let info_id = info.id.clone().unwrap();
            let roles = info.roles.clone().unwrap();
            for role in roles {
                let role_copy = role.clone();
                let rel = UserInfoRole::find_by_role_id_and_user_info_id(
                    &role_copy.hty_role_id.unwrap(),
                    &role_copy.user_app_info_id.unwrap(),
                    conn,
                )?;
                let _ = UserInfoRole::delete_by_id(&rel.the_id, conn);
            }
            let _ = UserAppInfo::delete(&info_id, conn);
        }
        Ok(())
    };
    let params = HashMap::new();
    let _ = exec_read_write_task(
        Box::new(task_delete),
        Some(params),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}

async fn notify(
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
    push_info: Json<PushInfo>,
) -> Json<HtyResponse<()>> {
    debug!("notify -> starts");
    let info_push = push_info.0;
    match notifications::raw_notify(info_push, host, db_pool).await {
        Ok(res) => {
            debug!("notify -> success to notify!");
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("notify -> failed to notify, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}


async fn update_hty_resource(
    _sudoer: HtySudoerTokenHeader,
    _auth: AuthorizationHeader,
    _host: HtyHostHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(resource): Json<ReqHtyResource>,
) -> Json<HtyResponse<ReqHtyResource>> {
    debug!("update_hty_resource -> {:?}", resource);
    match raw_update_hty_resource(resource, db_conn, db_pool).await {
        Ok(updated_resource) => {
            debug!(
                "update_hty_resource -> success to update hty_resource, updated_resource : {:?}!",
                updated_resource
            );
            wrap_json_ok_resp(updated_resource)
        }
        Err(e) => {
            error!(
                "update_hty_resource -> failed to update hty_resource, e: {}",
                e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::InternalErr,
                reason: Some(e.to_string()),
            })
        }
    }
}

async fn raw_update_hty_resource(
    resource: ReqHtyResource,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyResource> {
    let mut in_resource = HtyResource::strict_from(resource)?;
    in_resource.updated_at = Some(current_local_datetime());
    debug!("raw_update_hty_resource -> in_resource: {:?}", in_resource);
    Ok(HtyResource::update(
        &in_resource,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?
        .to_req())
}

async fn create_hty_resource(
    _sudoer: HtySudoerTokenHeader,
    auth: AuthorizationHeader,
    host: HtyHostHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    resource: Json<ReqHtyResource>,
) -> Json<HtyResponse<String>> {
    debug!("create_hty_resource -> starts");
    match raw_create_hty_resource(auth, host, resource, db_conn, db_pool).await {
        Ok(hty_resource_id) => {
            debug!(
                "create_hty_resource -> success to create hty resource, resource id: {:?}!",
                hty_resource_id
            );
            wrap_json_ok_resp(hty_resource_id)
        }
        Err(e) => {
            error!(
                "create_hty_resource -> failed to create hty resource, e: {}",
                e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::InternalErr,
                reason: Some(e.to_string()),
            })
        }
    }
}

async fn raw_create_hty_resource(
    auth: AuthorizationHeader,
    host: HtyHostHeader,
    resource: Json<ReqHtyResource>,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    match HtyApp::find_by_domain(&*host, extract_conn(fetch_db_conn(&db_pool)?).deref_mut()) {
        Ok(app) => {
            let hty_app_id = app.app_id;
            debug(
                format!(
                    "raw_create_hty_resource -> hty_app_id / #{:?}",
                    hty_app_id.clone()
                )
                    .as_str(),
            );
            match jwt_decode_token(&(*auth).clone()) {
                Ok(token) => {
                    debug(
                        format!("raw_create_hty_resource -> token / #{:?}", token.clone()).as_str(),
                    );

                    match HtyUser::find_by_hty_id(
                        &token.hty_id.clone().unwrap()[..],
                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    ) {
                        Ok(user) => {
                            debug(
                                format!("raw_create_hty_resource -> user / #{:?}", user.clone())
                                    .as_str(),
                            );

                            let mut req_resource = resource.clone().0;

                            req_resource.app_id = Some(hty_app_id.clone());
                            req_resource.created_by = Some(user.hty_id.clone());
                            req_resource.hty_resource_id = Some(uuid());

                            match HtyResource::strict_from(req_resource.clone()) {
                                Ok(in_resource) => {
                                    debug(
                                        format!(
                                            "raw_create_hty_resource -> in_resource / {:?}",
                                            in_resource
                                        )
                                            .as_str(),
                                    );

                                    match HtyResource::create(
                                        &in_resource,
                                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                                    ) {
                                        Ok(resource) => {
                                            debug(format!("raw_create_hty_resource HtyResource::create ok -> host: #{:?} / token: #{:?} / created resource: #{:?}", (*host).clone(), token.clone(), resource.clone()).as_str());
                                            Ok(resource.clone().hty_resource_id)
                                        }
                                        Err(e) => {
                                            debug(format!("raw_create_hty_resource HtyResource::create err -> host: #{:?} / token: #{:?} / in_resource: #{:?} / err: #{:?}", (*host).clone(), token.clone(), in_resource.clone(), e).as_str());
                                            Err(anyhow!(e))
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug(format!("raw_create_hty_resource HtyResource::from err -> host: #{:?} / token: #{:?} / in_resource: #{:?} / err: #{:?}", (*host).clone(), token.clone(), req_resource.clone(), e).as_str());
                                    Err(anyhow!(e))
                                }
                            }
                        }
                        Err(e) => {
                            debug(format!("raw_create_hty_resource HtyUser::find_by_id err -> host: #{:?} / token: #{:?} / err: #{:?}", (*host).clone(), token.clone(), e).as_str());
                            Err(anyhow!(e))
                        }
                    }
                }
                Err(e) => {
                    debug(format!("raw_create_hty_resource jwt_decode *auth err -> host: #{:?} / auth: #{:?} / err: #{:?}", (*host).clone(), (*auth).clone(), e.clone()).as_str());
                    Err(anyhow!(e))
                }
            }
        }
        Err(e) => {
            debug(format!("raw_create_hty_resource HtyApp::find_by_domain err -> host: #{:?} / auth: #{:?} / err: #{:?}", (*host).clone(), (*auth).clone(), e).as_str());
            Err(anyhow!(e))
        }
    }
}

async fn update_user_group(
    _auth: AuthorizationHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_user_group): Json<ReqHtyUserGroup>,
) -> Json<HtyResponse<()>> {
    match raw_update_user_group(&req_user_group, db_pool) {
        Ok(res) => {
            debug!(
                "update_user_group -> success to update user group : {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("update_user_group -> failed to update users, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_update_user_group(
    req_user_group: &ReqHtyUserGroup,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    if req_user_group.app_id.is_none()
        || req_user_group.group_type.is_none()
        || req_user_group.group_name.is_none()
        || req_user_group.id.is_none()
    {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("app_id or group_type or group_name or id is none".into()),
        }));
    };

    let id_user_group = req_user_group.id.clone().unwrap();

    let mut db_user_group = HtyUserGroup::find_by_id(
        &id_user_group,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    if req_user_group.users.is_some() {
        db_user_group.users = req_user_group.users.clone();
    }

    db_user_group.group_type = req_user_group.group_type.clone().unwrap();
    db_user_group.app_id = req_user_group.app_id.clone().unwrap();
    db_user_group.group_name = req_user_group.group_name.clone().unwrap();
    db_user_group.group_desc = req_user_group.group_desc.clone();
    db_user_group.parent_id = req_user_group.parent_id.clone();
    db_user_group.owners = req_user_group.owners.clone();

    debug!(
        "update_user_group -> ready to update : {:?}!",
        db_user_group
    );

    let _ = HtyUserGroup::update(
        &db_user_group,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}

async fn create_user_group(
    auth: AuthorizationHeader,
    host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_user_group): Json<ReqHtyUserGroup>,
) -> Json<HtyResponse<String>> {
    debug!("create_user_group -> {:?}", &req_user_group);
    match raw_create_user_group(&req_user_group, host, auth, db_pool) {
        Ok(res) => {
            debug!(
                "create_user_group -> success to create user group : {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("create_user_group -> failed to create users, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_user_group(
    req_user_group: &ReqHtyUserGroup,
    host: HtyHostHeader,
    _auth: AuthorizationHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    if req_user_group.group_type.is_none() || req_user_group.group_name.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("group_type or group_name is none".into()),
        }));
    };
    // let id_user = jwt_decode_token(&(*auth).clone())?.hty_id.unwrap();
    let in_app = get_app_from_host(
        (*host).clone(),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let in_user_group = HtyUserGroup {
        id: uuid(),
        users: req_user_group.users.clone(),
        group_type: req_user_group.group_type.clone().unwrap(),
        created_at: Some(current_local_datetime()),
        created_by: req_user_group.created_by.clone(),
        app_id: in_app.app_id,
        group_name: req_user_group.group_name.clone().unwrap(),
        is_delete: false,
        group_desc: req_user_group.group_desc.clone(),
        parent_id: req_user_group.parent_id.clone(),
        owners: req_user_group.owners.clone(),
    };
    let res = HtyUserGroup::create(
        &in_user_group,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(res.id)
}

async fn update_template_data(
    _auth: AuthorizationHeader,
    _host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_template_data): Json<ReqHtyTemplateData<String>>,
) -> Json<HtyResponse<String>> {
    match raw_update_template_data(&req_template_data, db_pool) {
        Ok(res) => {
            debug!(
                "update_template_data -> success to update template data: {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "update_template_data -> failed to update template data, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_update_template_data(
    req_template_data: &ReqHtyTemplateData<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    if req_template_data.id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("id is none".into()),
        }));
    };
    let id_template_data = req_template_data.id.clone().unwrap();
    let mut db_template_data = HtyTemplateData::find_by_id(
        &id_template_data,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    if req_template_data.template_id.is_some() {
        db_template_data.template_id = req_template_data.template_id.clone().unwrap();
    }
    if req_template_data.app_id.is_some() {
        db_template_data.app_id = req_template_data.app_id.clone().unwrap();
    }
    db_template_data.template_val = req_template_data.template_val.clone();
    db_template_data.template_text = req_template_data.template_text.clone();
    let _ = HtyTemplateData::update(
        &db_template_data,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(id_template_data)
}

async fn update_template(
    _auth: AuthorizationHeader,
    _host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_template): Json<ReqHtyTemplate<String>>,
) -> Json<HtyResponse<String>> {
    match raw_update_template(&req_template, db_pool) {
        Ok(res) => {
            debug!("update_template -> success to update template : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("update_template -> failed to update template, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_update_template(
    req_template: &ReqHtyTemplate<String>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    if req_template.id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("id is none".into()),
        }));
    };
    let id_template = req_template.id.clone().unwrap();
    let mut db_template = HtyTemplate::find_by_id(
        &id_template,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    if req_template.template_key.is_some() {
        db_template.template_key = req_template.template_key.clone().unwrap();
    }
    db_template.template_desc = req_template.template_desc.clone();
    let _ = HtyTemplate::update(
        &db_template,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    Ok(id_template)
}

async fn create_template_data(
    auth: AuthorizationHeader,
    _host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_template_data): Json<ReqHtyTemplateData<String>>,
) -> Json<HtyResponse<String>> {
    match raw_create_template_data(&req_template_data, auth, db_pool) {
        Ok(res) => {
            debug!(
                "create_template_data -> success to create template data : {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "create_template_data -> failed to create template data, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_template_data(
    req_template_data: &ReqHtyTemplateData<String>,
    auth: AuthorizationHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    if req_template_data.app_id.is_none() || req_template_data.template_id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("app_id or template_id is none".into()),
        }));
    };
    let id_user = jwt_decode_token(&(*auth).clone())?.hty_id.unwrap();
    let in_template_data = HtyTemplateData {
        id: uuid(),
        app_id: req_template_data.app_id.clone().unwrap(),
        template_id: req_template_data.template_id.clone().unwrap(),
        template_val: req_template_data.template_val.clone(),
        template_text: req_template_data.template_text.clone(),
        created_at: current_local_datetime(),
        created_by: id_user,
    };
    let res = HtyTemplateData::<String>::create::<String>(
        &in_template_data,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(res.id)
}

async fn create_template(
    auth: AuthorizationHeader,
    _host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_template): Json<ReqHtyTemplate<String>>,
) -> Json<HtyResponse<String>> {
    match raw_create_template(&req_template, auth, db_pool) {
        Ok(res) => {
            debug!("create_template -> success to create template : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("create_template -> failed to creare template, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_template(
    req_template: &ReqHtyTemplate<String>,
    auth: AuthorizationHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    if req_template.template_key.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("template_key is none".into()),
        }));
    };
    let id_user = jwt_decode_token(&(*auth).clone())?.hty_id.unwrap();
    let in_template = HtyTemplate {
        id: uuid(),
        template_key: req_template.template_key.clone().unwrap(),
        created_at: current_local_datetime(),
        created_by: id_user,
        template_desc: req_template.template_desc.clone(),
    };
    let res = HtyTemplate::create(
        &in_template,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(res.id)
}

async fn get_templates(
    _auth: AuthorizationHeader,
    _host: HtyHostHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyTemplate<String>>>> {
    match raw_get_templates(db_pool) {
        Ok(res) => {
            debug!("get_templates -> success to get templates : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("get_templates -> failed to get templates, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_get_templates(db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyTemplate<String>>> {
    let mut res = vec![];
    let template_vec = HtyTemplate::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    for template_item in template_vec {
        let template_data = template_item.find_linked_template_data::<String>(
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        let req_template_data_vec = template_data
            .iter()
            .map(|item| ReqHtyTemplateData {
                id: Some(item.id.clone()),
                app_id: Some(item.app_id.clone()),
                template_id: Some(item.template_id.clone()),
                template_val: item.template_val.clone(),
                template_text: item.template_text.clone(),
                created_at: Some(item.created_at.clone()),
                created_by: Some(item.created_by.clone()),
            })
            .collect();
        let res_item = ReqHtyTemplate {
            id: Some(template_item.id.clone()),
            template_key: Some(template_item.template_key.clone()),
            created_at: Some(template_item.created_at.clone()),
            created_by: Some(template_item.created_by.clone()),
            template_desc: template_item.template_desc.clone(),
            datas: Some(req_template_data_vec),
        };
        res.push(res_item)
    }
    Ok(res)
}

async fn get_all_tags_of_the_user(auth: AuthorizationHeader, _sudoer: HtySudoerTokenHeader, host: HtyHostHeader, State(db_pool): State<Arc<DbState>>) -> Json<HtyResponse<Vec<ReqHtyTag>>> {
    match raw_get_all_tags_of_the_user(auth, host, db_pool) {
        Ok(res) => {
            debug!("get_all_tags_of_the_user -> success to find tags : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("get_all_tags_of_the_user -> failed to find tags, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_get_all_tags_of_the_user(auth: AuthorizationHeader, host: HtyHostHeader, db_pool: Arc<DbState>) -> anyhow::Result<Vec<ReqHtyTag>> {
    let db_tags = get_all_db_tags_of_the_user(auth, host, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let res: Vec<ReqHtyTag> = db_tags.into_iter().map(|tag| {
        tag.to_req()
    }).collect();

    Ok(res)
}

fn get_all_db_tags_of_the_user(auth: AuthorizationHeader, host: HtyHostHeader, conn: &mut PgConnection) -> anyhow::Result<Vec<HtyTag>> {
    let domain = (*host).clone();
    debug!("get_all_db_tags_of_the_user -> domain: {:?}", domain);

    let app = HtyApp::find_by_domain(&domain, conn)?;
    debug!("get_all_db_tags_of_the_user -> app: {:?}", app);

    let id_user = jwt_decode_token(&(*auth).clone())?.hty_id.unwrap();
    debug!("get_all_db_tags_of_the_user -> id_user: {:?}", id_user);

    let user_app_info = UserAppInfo::find_by_hty_id_and_app_id(&id_user, &app.app_id, conn)?;
    debug!("get_all_db_tags_of_the_user -> user_app_info: {:?}", user_app_info);

    let role_vec = user_app_info.find_linked_roles(conn)?;
    debug!("get_all_db_tags_of_the_user -> find_linked_roles: {:?}", role_vec);

    let mut tag_vec = HtyTag::find_all_by_ref_id(&user_app_info.clone().id, conn)?;
    debug!("get_all_db_tags_of_the_user -> user_tags: {:?}", tag_vec);

    for role in role_vec {
        let mut linked_tag = HtyTag::find_all_by_ref_id(&role.hty_role_id, conn)?;
        debug!("get_all_db_tags_of_the_user -> role_tags: {:?}", linked_tag);
        tag_vec.append(&mut linked_tag)
    }
    debug!("get_all_db_tags_of_the_user -> user_with_roles_tags: {:?}", tag_vec);

    Ok(tag_vec)
}

//
async fn get_user_groups_of_current_user(
    Query(params): Query<HashMap<String, String>>,
    auth: AuthorizationHeader,
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<ReqHtyUserGroup>>> {
    debug!("get_user_groups_of_current_user -> TOTAL START {:?}", current_local_datetime());

    let some_is_delete = get_some_from_query_params::<bool>("is_delete", &params);

    match raw_get_user_groups_of_current_user(auth, &some_is_delete, db_pool) {
        Ok(res) => {
            debug!("raw_get_user_groups_of_current_user -> success to find user group : {:?}!", res);
            let res = wrap_json_ok_resp(res);
            debug!("get_user_groups_of_current_user -> TOTAL END {:?}", current_local_datetime());
            res
        }
        Err(e) => {
            error!("raw_get_user_groups_of_current_user -> failed to find users, e: {}", e);
            let res = wrap_json_anyhow_err(e);
            debug!("get_user_groups_of_current_user -> TOTAL END {:?}", current_local_datetime());
            res
        }
    }
}

fn raw_get_user_groups_of_current_user(
    auth: AuthorizationHeader,
    some_is_delete: &Option<bool>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Vec<ReqHtyUserGroup>> {
    let id_user = jwt_decode_token(&(*auth).clone())?.hty_id.unwrap();

    debug!("raw_get_user_groups_of_current_user -> find_by_created_by_or_users START: {:?}", current_local_datetime());
    let res = HtyUserGroup::find_by_created_by_or_users(
        &id_user,
        some_is_delete,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    debug!("raw_get_user_groups_of_current_user -> find_by_created_by_or_users END: {:?} / {:?}", current_local_datetime(), res);

    debug!("raw_get_user_groups_of_current_user -> req_res START: {:?}", current_local_datetime());
    let req_res = res
        .iter()
        .map(|item| item.to_req())
        .collect();

    debug!("raw_get_user_groups_of_current_user -> req_res END: {:?} / {:?}", current_local_datetime(), req_res);
    Ok(req_res)
}


async fn delete_hty_resource_by_id(sudoer: HtySudoerTokenHeader,
                                   host: HtyHostHeader,
                                   Path(id_hty_resource): Path<String>,
                                   conn: db::DbConn) -> Json<HtyResponse<()>> {
    let _ = raw_delete_hty_resource_by_id(id_hty_resource, sudoer, host, extract_conn(conn).deref_mut()).await;
    wrap_json_ok_resp(())
}

async fn raw_delete_hty_resource_by_id(id_hty_resource: String, sudoer: HtySudoerTokenHeader, host: HtyHostHeader, conn: &mut PgConnection) -> anyhow::Result<()> {
    debug!("raw_delete_hty_resource_by_id -> id: {:?}", id_hty_resource);
    let hty_resource = HtyResource::find_by_id(id_hty_resource.as_str(), conn)?;

    debug!("raw_delete_hty_resource_by_id -> hty_resource: {:?}", hty_resource);
    // if hty_resource.url.is_some() {
    let filename = extract_filename_from_url(&hty_resource.url);
    debug!("raw_delete_hty_resource_by_id -> filename: {:?}", filename);
    let _ = upyun_delete_by_filename(&filename, &sudoer.0, &host.0).await?;
    // }

    let _ = HtyResource::delete_by_id(&hty_resource.hty_resource_id, conn);
    Ok(())
}


async fn find_users_by_app_id(
    _sudoer: HtySudoerTokenHeader,
    _host: HtyHostHeader,
    Path(app_id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqHtyUser>>> {
    debug!("find_users_by_app_id -> {:?}", app_id);

    match HtyUser::find_all_by_app_id(&app_id, extract_conn(conn).deref_mut()) {
        Ok(in_users) => {
            debug!(
                "find_users_by_app_id -> success to find users, users: {:?}!",
                in_users
            );
            wrap_json_ok_resp(HtyUser::to_req_users(&in_users))
        }
        Err(e) => {
            error!("find_users_by_app_id -> failed to find users, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

async fn find_app_with_roles(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<ReqHtyApp>> {
    debug!("find_app_with_roles -> starts");
    match raw_find_app_with_roles(&id, &db_pool) {
        Ok(app) => {
            debug!(
                "find_app_with_roles -> success to find app by id: {}, app: {:?}!",
                id, app
            );
            wrap_json_ok_resp(app)
        }
        Err(e) => {
            error!("find_app_with_roles -> failed to find apps, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_app_with_roles(id: &String, db_pool: &Arc<DbState>) -> anyhow::Result<ReqHtyApp> {
    let hty_app = HtyApp::find_by_id(&id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let roles = hty_app.find_linked_roles(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let gonggaos = HtyGongGao::find_by_app_id(
        &hty_app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    // todo fill in `tags`
    Ok(ReqHtyApp {
        app_id: Some(hty_app.app_id),
        wx_id: hty_app.wx_id,
        wx_secret: hty_app.wx_secret,
        domain: hty_app.domain,
        app_desc: hty_app.app_desc,
        app_status: Some(hty_app.app_status),
        role_ids: None,
        roles: Some(roles),
        gonggaos: Some(gonggaos),
        tags: None, // todo -> fill in tags
        pubkey: hty_app.pubkey,
        privkey: hty_app.privkey,
        needs_refresh: None,
        is_wx_app: hty_app.is_wx_app.clone(),
    })
}

async fn set_hty_resource_compression_processed(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<()>> {
    debug!("set_hty_resource_compression_processed -> starts");
    match raw_set_hty_resource_compression_processed(&id, &db_pool) {
        Ok(_) => {
            debug!(
                "set_hty_resource_compression_processed -> success {}",
                id
            );
            wrap_json_ok_resp(())
        }
        Err(e) => {
            error!("set_hty_resource_compression_processed -> failed e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_set_hty_resource_compression_processed(id: &String, db_pool: &Arc<DbState>) -> anyhow::Result<HtyResource> {
    let mut hty_resource = HtyResource::find_by_id(id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    hty_resource.compress_processed = Some(true);
    HtyResource::update(&hty_resource, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())
}

async fn disable_app_from_to(
    _sudoer: HtySudoerTokenHeader,
    Path(disable_id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<()>> {
    debug!("disable_app_from_to -> starts");
    match raw_disable_app_from_to(&disable_id, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!(
                "disable_app_from_to -> success disable app_from_to: {:?}!",
                ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "disable_app_from_to -> failed to disable app_from_to, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_disable_app_from_to(disable_id: &String, conn: &mut PgConnection) -> anyhow::Result<()> {
    let _ = AppFromTo::disable_by_id(disable_id, conn)?;
    Ok(())
}

async fn enable_app_from_to(
    _sudoer: HtySudoerTokenHeader,
    Path(enable_id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<()>> {
    debug!("enable_app_from_to -> starts");
    match raw_enable_app_from_to(&enable_id, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!(
                "enable_app_from_to -> success enable app_from_to: {:?}!",
                ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "enable_app_from_to -> failed to enable app_from_to, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_enable_app_from_to(enable_id: &String, conn: &mut PgConnection) -> anyhow::Result<()> {
    let _ = AppFromTo::enable_by_id(enable_id, conn)?;
    Ok(())
}

async fn create_app_from_to(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Json(req_from_to): Json<ReqAppFromTo>,
) -> Json<HtyResponse<String>> {
    debug!("create_app_from_to -> starts");
    match raw_create_app_from_to(&req_from_to, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!(
                "create_app_from_to -> success create app_from_to: {:?}!",
                ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "create_app_from_to -> failed to create app_from_to, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_app_from_to(
    req_from_to: &ReqAppFromTo,
    conn: &mut PgConnection,
) -> anyhow::Result<String> {
    if req_from_to.from_app_id.is_none() || req_from_to.to_app_id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("from_app_id or to_app_id".into()),
        }));
    }
    let in_from_to = AppFromTo {
        id: uuid(),
        from_app_id: req_from_to.from_app_id.clone().unwrap(),
        to_app_id: req_from_to.to_app_id.clone().unwrap(),
        is_enabled: true,
    };
    let res = AppFromTo::create(&in_from_to, conn)?;
    Ok(res.id)
}

async fn get_all_app_from_tos(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqAppFromTo>>> {
    debug!("get_all_app_from_tos -> starts");
    match raw_get_all_app_from_tos(extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!(
                "get_all_app_from_tos -> success find all app_from_to: {:?}!",
                ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "get_all_app_from_tos -> failed to find app_from_to, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_get_all_app_from_tos(conn: &mut PgConnection) -> anyhow::Result<Vec<ReqAppFromTo>> {
    let res = AppFromTo::all_to_reqs(&AppFromTo::find_all(conn)?);
    Ok(res)
}

async fn find_to_apps_by_domain(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqAppFromTo>>> {
    debug!("find_to_app_by_domain -> starts");
    match raw_find_to_apps_by_domain(host, extract_conn(conn).deref_mut()) {
        Ok(app) => {
            debug!("find_to_app_by_domain -> success find to_app: {:?}!", app);
            wrap_json_ok_resp(app)
        }
        Err(e) => {
            error!("find_to_app_by_domain -> failed to find to_app, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_to_apps_by_domain(
    host: HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<ReqAppFromTo>> {
    let domain = (*host).clone();
    let from_app = HtyApp::find_by_domain(&domain, conn)?;
    let to_apps = AppFromTo::find_all_active_to_apps_by_from_app(&from_app.app_id, conn)?;
    Ok(AppFromTo::all_to_reqs(&to_apps))
}

async fn create_or_update_user_with_info(
    _auth: AuthorizationHeader,
    hty_host: HtyHostHeader,
    _db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(user_with_info): Json<(Option<ReqHtyUser>, Option<ReqUserAppInfo>)>,
) -> Json<HtyResponse<(String, String)>> {
    debug!("create_or_update_user_with_info -> starts");

    match raw_create_or_update_user_with_info(hty_host, user_with_info, db_pool) {
        Ok(res) => {
            debug!("create_or_update_user_with_info -> success : {:?}!", res);
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!("create_or_update_user_with_info -> failed e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_user_with_info(
    hty_host: HtyHostHeader,
    user_with_info: (Option<ReqHtyUser>, Option<ReqUserAppInfo>),
    db_pool: Arc<DbState>,
) -> anyhow::Result<(String, String)> {
    let app_domain = (*hty_host).clone();

    let (user, info) =
        verify_user_with_info(&user_with_info, &db_pool, &app_domain)?;

    debug!("raw_create_or_update_user_with_info -> raw_create_or_update_user_with_info_tx");

    raw_create_or_update_user_with_info_tx(&user, &info, db_pool)
}

fn verify_user_with_info(
    user_with_info: &(Option<ReqHtyUser>, Option<ReqUserAppInfo>),
    db_pool: &Arc<DbState>,
    app_domain: &String,
) -> anyhow::Result<(Option<ReqHtyUser>, Option<ReqUserAppInfo>)> {
    debug!(
        "verify_user_with_info -> APP_DOMAIN: {}",
        app_domain
    );

    debug!(
        "verify_user_with_info -> user_with_info: {:?}",
        user_with_info
    );

    let mut params = HashMap::new();

    params.insert(
        "params".to_string(),
        (user_with_info.clone(), app_domain.clone()),
    );

    let task = move |in_params: Option<
        HashMap<String, ((Option<ReqHtyUser>, Option<ReqUserAppInfo>), String)>,
    >,
                     conn: &mut PgConnection|
                     -> anyhow::Result<(Option<ReqHtyUser>, Option<ReqUserAppInfo>)> {
        let (user_with_info, app_domain) = in_params
            .ok_or(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some(
                    "verify_user_with_info(): in_params is empty!!!".to_string(),
                ),
            })?
            .get("params")
            .ok_or(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some(
                    "verify_user_with_info(): in_params is empty!!!".to_string(),
                ),
            })?
            .clone();

        let (user, mut info) = user_with_info.clone();

        if user.is_none() && info.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("empty request".into()),
            }));
        };

        if !info.is_none() {
            let in_info = info.clone().unwrap();
            let in_app_id = match in_info.app_id {
                Some(app_id) => app_id,
                None => HtyApp::find_by_domain(&app_domain, conn)?.app_id,
            };

            verify_username(&user, &info, &in_app_id, conn)?;

            info.as_mut()
                .map(|_info| _info.app_id = Some(in_app_id.clone()));

            match HtyApp::verify_exist_by_id(&in_app_id, conn) {
                Ok(ok) => {
                    if ok {
                        debug!(
                            "verify_user_with_info() -> has this hty_app : {:?}, {:?}",
                            in_app_id, ok
                        );
                    } else {
                        return Err(anyhow!(HtyErr {
                            code: HtyErrCode::NullErr,
                            reason: Some(format!("don't have this htyapp -> {}", &in_app_id))
                        }));
                    }
                }
                Err(e) => {
                    return Err(anyhow!(e));
                }
            }
        }
        Ok((user, info))
    };

    return match exec_read_write_task(
        Box::new(task),
        Some(params),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    ) {
        Ok(res) => Ok(res),
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some(format!(
                "verify_user_with_info -> met err {}",
                e.to_string()
            )),
        })),
    };
}

fn raw_create_or_update_user_with_info_tx(
    user: &Option<ReqHtyUser>,
    info: &Option<ReqUserAppInfo>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<(String, String)> {
    let mut params = HashMap::new();

    params.insert("params".to_string(), (user.clone(), info.clone()));

    let task = move |in_params: Option<
        HashMap<String, (Option<ReqHtyUser>, Option<ReqUserAppInfo>)>,
    >,
                     conn: &mut PgConnection|
                     -> anyhow::Result<(String, String)> {
        let (some_user, info) = in_params
            .ok_or(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("raw_create_or_update_user_with_info_tx(): in_params is empty!!!".to_string()),
            })?
            .get("params")
            .ok_or(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("raw_create_or_update_user_with_info_tx(): in_params is empty!!!".to_string()),
            })?
            .clone();
        let mut res_user_id = String::new();
        let mut res_user_info_id = String::new();
        if !some_user.is_none() {
            if !some_user.clone().unwrap().hty_id.is_none() {
                let in_user = some_user.unwrap().to_hty_user()?;
                res_user_id = HtyUser::update(&in_user, conn)?.hty_id;
            } else {
                let c_user = some_user.clone().unwrap();
                let in_user = HtyUser {
                    hty_id: uuid(),
                    union_id: c_user.union_id,
                    enabled: true,
                    created_at: Some(current_local_datetime()),
                    real_name: c_user.real_name,
                    sex: c_user.sex,
                    mobile: c_user.mobile,
                    settings: c_user.settings,
                };
                debug!("raw_create_or_update_user_with_info_tx -> in_user: {:?}", in_user);
                res_user_id = HtyUser::create(&in_user, conn)?.hty_id;
            }
        }
        if !info.is_none() {
            if !info.clone().unwrap().id.is_none() {
                let in_info = info.clone().unwrap().to_user_app_info()?;
                res_user_info_id = UserAppInfo::update(&in_info, conn)?.id;
            } else {
                let mut tem_hty_id = info.clone().unwrap().hty_id;
                if tem_hty_id.is_none() {
                    if res_user_id.is_empty() {
                        return Err(anyhow!(HtyErr {
                            code: HtyErrCode::WebErr,
                            reason: Some("hty_id not exits for this request".into()),
                        }));
                    }
                    tem_hty_id = Some(res_user_id.clone());
                }

                let c_info = info.clone().unwrap();
                let is_exist = UserAppInfo::verify_exist_by_app_id_and_hty_id(&tem_hty_id.clone().unwrap(), &c_info.app_id.clone().unwrap(), conn)?;
                if is_exist {
                    return Err(anyhow!(HtyErr {
                            code: HtyErrCode::WebErr,
                            reason: Some("there is existing record for the created user_app_info".into()),
                        }));
                }

                let in_info = UserAppInfo {
                    hty_id: tem_hty_id.unwrap(),
                    app_id: c_info.app_id,
                    openid: c_info.openid,
                    is_registered: true,
                    id: uuid(),
                    username: c_info.username,
                    password: c_info.password,
                    meta: c_info.meta,
                    created_at: c_info.created_at,
                    teacher_info: c_info.teacher_info,
                    student_info: c_info.student_info,
                    reject_reason: c_info.reject_reason,
                    needs_refresh: c_info.needs_refresh,
                    avatar_url: c_info.avatar_url,
                };
                res_user_info_id = UserAppInfo::create(&in_info, conn)?.id;
            }
        }
        Ok((res_user_id, res_user_info_id))
    };

    return match exec_read_write_task(
        Box::new(task),
        Some(params),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    ) {
        Ok(res) => Ok(res),
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some(format!(
                "raw_create_or_update_user_with_info_tx -> met err {}",
                e.to_string()
            )),
        })),
    };
}

fn verify_username(
    user: &Option<ReqHtyUser>,
    info: &Option<ReqUserAppInfo>,
    app_id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<()> {
    if info.is_none() {
        return Ok(());
    }

    let id_user;
    if user.is_none() {
        id_user = info.clone().unwrap().hty_id;
    } else {
        id_user = user.clone().unwrap().hty_id;
    }

    let in_info = info.clone().unwrap();
    let in_username = in_info.username.clone();

    if !in_username.is_none() {
        if in_username.clone().unwrap() != "" {
            let res =
                UserAppInfo::verify_unique_username_by_app_id(&in_info.clone(), app_id, conn)?;
            if res {
                if !id_user.is_none() {
                    // 如果和自己重名，则不是问题，此场景适用于update_user的username之时。
                    let my_info = UserAppInfo::find_by_username_and_app_id(
                        &in_username.clone().unwrap(),
                        app_id,
                        conn,
                    )?;
                    if Some(my_info.hty_id) == id_user {
                        return Ok(());
                    }
                } else {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::InternalErr,
                        reason: Some(format!(
                            "existing username -> {}",
                            &in_username.clone().unwrap()
                        ))
                    }));
                }
            }
        }
    }

    Ok(())
}

async fn update_needs_refresh_for_app(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Json(hty_app): Json<ReqHtyApp>,
) -> Json<HtyResponse<ReqHtyApp>> {
    debug!("update_needs_refresh_for_app -> starts -> {:?}", &hty_app);
    match raw_update_needs_refresh_for_app(&hty_app, extract_conn(conn).deref_mut()) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!("update_needs_refresh_for_app -> failed to update, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_update_needs_refresh_for_app(
    hty_app: &ReqHtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyApp> {
    let in_app = HtyApp::find_by_id(&hty_app.app_id.clone().unwrap(), conn)?;
    let needs_refresh = hty_app.needs_refresh.clone();

    let all_infos = in_app.all_user_infos(conn)?;

    for mut info in all_infos {
        info.needs_refresh = needs_refresh.clone();

        let updated_info = UserAppInfo::update(&info, conn)?;
        debug!(
            "raw_update_needs_refresh_for_app -> updated_info -> {:?}",
            &updated_info
        );
    }

    Ok(hty_app.clone())
}

async fn create_or_update_apps_with_roles(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Json(hty_app): Json<ReqHtyApp>,
) -> Json<HtyResponse<ReqHtyApp>> {
    debug!("create_or_update_apps_with_roles -> starts");
    match raw_create_or_update_apps_with_roles(&hty_app, extract_conn(conn).deref_mut()) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!(
                "create_or_update_apps_with_roles -> failed to create/update roles, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_apps_with_roles(
    hty_app: &ReqHtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyApp> {
    let mut req_app = hty_app.clone();

    if req_app.app_status.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("app_status is none".into()),
        }));
    };

    let in_app;
    let mut is_new_app = false;

    if req_app.app_id.is_none() {
        req_app.app_id = Some(uuid());
        is_new_app = true;
    }

    let cloned = req_app.clone();

    in_app = HtyApp {
        app_id: cloned.app_id.unwrap(),
        wx_secret: cloned.wx_secret,
        domain: cloned.domain,
        app_status: cloned.app_status.unwrap(),
        app_desc: cloned.app_desc,
        pubkey: cloned.pubkey,
        privkey: cloned.privkey,
        wx_id: cloned.wx_id,
        is_wx_app: cloned.is_wx_app,
    };

    let exist = HtyApp::verify_exist_by_id(&req_app.app_id.clone().unwrap(), conn)?;

    if exist {
        HtyApp::update(&in_app, conn)?;
    } else {
        HtyApp::create(&in_app, conn)?;
    }

    // 1. 如果传进来为空，则删除所有roles
    // 2. 如果不为空，更新为传进来的关系
    if req_app.role_ids.clone().is_none() {
        if !is_new_app {
            let _ = AppRole::delete_all_by_app_id(&req_app.app_id.clone().unwrap(), conn)?;
        }
    } else {
        // 先删后增
        // todo: 未来可能重构逻辑
        let _ = AppRole::delete_all_by_app_id(&req_app.app_id.clone().unwrap(), conn)?;
        let roles = req_app.role_ids.clone().unwrap();
        for id_role in roles {
            let entry = AppRole {
                the_id: uuid().clone(),
                app_id: in_app.clone().app_id,
                role_id: id_role.clone(),
            };
            let _ = AppRole::create(&entry, conn)?;
        }
    }

    Ok(req_app)
}

async fn find_tongzhi_by_id(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Path(tongzhi_id): Path<String>,
) -> Json<HtyResponse<Option<ReqHtyTongzhi>>> {
    debug!("find_tongzhi_by_id -> starts");

    match raw_find_tongzhi_by_id(&tongzhi_id, extract_conn(conn).deref_mut()) {
        Ok(some_tz) => {
            if some_tz.is_some() {
                let req_tz = some_tz.unwrap().to_req();

                wrap_json_ok_resp(Some(req_tz))
            } else {
                wrap_json_ok_resp(None)
            }
        }

        Err(e) => {
            error!("find_tongzhi_by_id -> ERR: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_tongzhi_by_id(
    tongzhi_id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<Option<HtyTongzhi>> {
    let some_in_tongzhi = HtyTongzhi::find_by_id2(tongzhi_id, conn)?;
    Ok(some_in_tongzhi)
}

async fn create_tongzhi(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Json(req_tongzhi): Json<ReqHtyTongzhi>,
) -> Json<HtyResponse<String>> {
    debug!("create_tongzhi -> starts");
    match raw_create_tongzhi(&req_tongzhi, extract_conn(conn).deref_mut()) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!("create_tongzhi -> failed create tongzhi, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_tongzhi(
    req_tongzhi: &ReqHtyTongzhi,
    conn: &mut PgConnection,
) -> anyhow::Result<String> {
    let in_tongzhi = req_tongzhi.to_db_struct();
    let res = HtyTongzhi::create(&in_tongzhi, conn)?;
    Ok(res.tongzhi_id)
}

async fn delete_template(
    _sudoer: HtySudoerTokenHeader,
    Path(template_id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<()>> {
    debug!("delete_template -> template_id: {:?}", template_id);
    match raw_delete_template(&template_id, db_pool) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!("delete_template -> failed delete template, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_delete_template(template_id: &String, db_pool: Arc<DbState>) -> anyhow::Result<()> {
    let _ = HtyTemplate::delete_by_id(
        template_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}

async fn delete_template_data(
    _sudoer: HtySudoerTokenHeader,
    Path(template_data_id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<()>> {
    debug!(
        "delete_template_data -> template_data_id: {:?}",
        template_data_id
    );
    match raw_delete_template_data(&template_data_id, extract_conn(conn).deref_mut()) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!(
                "delete_template_data -> failed delete template data, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_delete_template_data(
    template_data_id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<()> {
    let _ = HtyTemplateData::<String>::delete_by_id(template_data_id, conn)?;
    Ok(())
}

async fn delete_tongzhi_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<String>> {
    debug!("delete_tongzhi_by_id -> id: {:?}", id);
    match raw_delete_tongzhi_by_id(&id, extract_conn(conn).deref_mut()) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!("delete_tongzhi_by_id -> failed delete tongzhi, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_delete_tongzhi_by_id(
    id_tongzhi: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<String> {
    let res = HtyTongzhi::delete_by_id(id_tongzhi, conn)?;
    Ok(res.tongzhi_id)
}

async fn create_or_update_userinfo_with_roles(
    _sudoer: HtySudoerTokenHeader,
    conn: db::DbConn,
    Json(hty_user_app_info): Json<ReqUserAppInfo>,
) -> Json<HtyResponse<ReqUserAppInfo>> {
    debug!("create_or_update_userinfo_with_roles -> starts");
    match raw_create_or_update_userinfo_with_roles(
        &hty_user_app_info,
        extract_conn(conn).deref_mut(),
    ) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!("create_or_update_userinfo_with_roles -> failed create or update userinfo with role, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_userinfo_with_roles(
    hty_user_app_info: &ReqUserAppInfo,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqUserAppInfo> {
    let req_user_app_info = hty_user_app_info.clone();
    let user_app_info = UserAppInfo::find_by_req_info(&req_user_app_info, conn)?;
    if user_app_info.app_id.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("app id can not be none".into()),
        }));
    }
    let app_roles: Vec<String> = HtyApp::find_by_id(&user_app_info.app_id.clone().unwrap(), conn)?
        .find_linked_roles(conn)?
        .iter()
        .map(|role| role.clone().hty_role_id)
        .collect();
    if req_user_app_info.roles.is_some() {
        let user_info_roles = req_user_app_info.roles.clone().unwrap();
        let user_info_role_ids: Vec<String> = user_info_roles
            .iter()
            .map(|item| item.hty_role_id.clone().unwrap())
            .collect();
        for user_info_role in user_info_roles.clone() {
            if !app_roles.contains(&user_info_role.hty_role_id.clone().unwrap()) {
                return Err(anyhow!(HtyErr {
                    code: HtyErrCode::InternalErr,
                    reason: Some("There exist unsupported role in the request".into()),
                }));
            }
        }
        for user_info_role in user_info_roles {
            match UserInfoRole::verify_exist_by_user_info_id_and_role_id(
                &user_app_info.id,
                &user_info_role.hty_role_id.clone().unwrap(),
                conn,
            ) {
                Ok(true) => continue,
                Ok(false) => {
                    let entry = UserInfoRole {
                        the_id: uuid(),
                        user_info_id: user_app_info.id.clone(),
                        role_id: user_info_role.hty_role_id.clone().unwrap().clone(),
                    };
                    UserInfoRole::create(&entry, conn)?;
                }
                Err(e) => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some(e.to_string()),
                    }));
                }
            }
        }
        let db_user_info_linked_roles =
            UserInfoRole::get_all_roles_by_user_info_id(&user_app_info.id, conn)?;
        for db_user_info_linked_role in db_user_info_linked_roles {
            if !user_info_role_ids.contains(&db_user_info_linked_role) {
                UserInfoRole::delete_by_role_id_and_user_info_id(
                    &db_user_info_linked_role,
                    &user_app_info.id,
                    conn,
                )?;
            }
        }
    }
    if req_user_app_info.roles.is_none() {
        UserInfoRole::delete_by_user_info_id(&user_app_info.id, conn)?;
    }

    Ok(req_user_app_info)
}

async fn create_or_update_roles(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(hty_role): Json<ReqHtyRole>,
) -> Json<HtyResponse<ReqHtyRole>> {
    debug!("create_or_update_roles -> starts");
    match raw_create_or_update_roles(&hty_role, db_pool) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!(
                "create_or_update_roles -> failed to create/update roles, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_roles(
    hty_role: &ReqHtyRole,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyRole> {
    let role = hty_role.clone();
    if role.labels.is_some() {
        for label in role.labels.clone().unwrap() {
            match HtyLabel::verify_exist_by_id(
                &label.hty_label_id.unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid label id exits".into()),
                    }));
                }
            }
        }
    }
    if role.actions.is_some() {
        for action in role.actions.clone().unwrap() {
            match HtyAction::verify_exist_by_id(
                &action.hty_action_id.unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid action id exits".into()),
                    }));
                }
            }
        }
    }
    if role.app_ids.is_some() {
        for app_id in role.app_ids.clone().unwrap() {
            match HtyApp::verify_exist_by_id(
                &app_id,
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid app id exits".into()),
                    }));
                }
            }
        }
    }
    if role.role_key.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("role name can not be none".into()),
        }));
    }
    if role.role_status.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("role status can not be none".into()),
        }));
    }

    let params = HashMap::new();
    // role_id is none, then create new role
    if role.hty_role_id.is_none() {
        let task_create_role = move |_in_params: Option<HashMap<String, ReqHtyRole>>,
                                     conn: &mut PgConnection|
                                     -> anyhow::Result<ReqHtyRole> {
            let mut res_role = role.clone();
            let in_role = HtyRole {
                hty_role_id: uuid(),
                role_key: role.role_key.clone().unwrap(),
                role_desc: role.role_desc.clone(),
                role_status: role.role_status.clone().unwrap(),
                style: role.style.clone(),
                role_name: role.role_name.clone(),
            };
            let created_role = HtyRole::create(&in_role, conn)?;
            res_role.hty_role_id = Some(created_role.hty_role_id.clone());
            if role.app_ids.is_some() {
                for app in role.app_ids.clone().unwrap() {
                    let in_app = AppRole {
                        the_id: uuid(),
                        role_id: in_role.hty_role_id.clone(),
                        app_id: app.clone(),
                    };
                    AppRole::create(&in_app, conn)?;
                }
            }
            if role.labels.is_some() {
                for label in role.labels.clone().unwrap() {
                    let in_label = RoleLabel {
                        the_id: uuid(),
                        role_id: in_role.hty_role_id.clone(),
                        label_id: label.hty_label_id.clone().unwrap(),
                    };
                    RoleLabel::create(&in_label, conn)?;
                }
            }
            if role.actions.is_some() {
                for action in role.actions.clone().unwrap() {
                    let in_action = RoleAction {
                        the_id: uuid(),
                        role_id: in_role.hty_role_id.clone(),
                        action_id: action.hty_action_id.clone().unwrap(),
                    };
                    RoleAction::create(&in_action, conn)?;
                }
            }
            return Ok(res_role.clone());
        };
        return match exec_read_write_task(
            Box::new(task_create_role),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            _ => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("hty role updated error ".into()),
            })),
        };
    }

    // role_id is some, then update existing role
    if role.hty_role_id.is_some() {
        let task_update_role = move |_in_params: Option<HashMap<String, ReqHtyRole>>,
                                     conn: &mut PgConnection|
                                     -> anyhow::Result<ReqHtyRole> {
            RoleLabel::delete_by_role_id(&role.hty_role_id.clone().unwrap(), conn)?;
            AppRole::delete_by_role_id(&role.hty_role_id.clone().unwrap(), conn)?;
            RoleAction::delete_by_role_id(&role.hty_role_id.clone().unwrap(), conn)?;
            match HtyRole::verify_exist_by_id(&role.hty_role_id.clone().unwrap(), conn) {
                Ok(true) => {
                    let update_role = HtyRole {
                        hty_role_id: role.hty_role_id.clone().unwrap(),
                        role_key: role.role_key.clone().unwrap(),
                        role_desc: role.clone().role_desc,
                        role_status: role.role_status.clone().unwrap(),
                        style: role.style.clone(),
                        role_name: role.role_name.clone(),
                    };
                    HtyRole::update(&update_role, conn)?;
                    if role.app_ids.is_some() {
                        for app_id in role.app_ids.clone().unwrap() {
                            match AppRole::verify_exist_by_app_id_and_role_id(
                                &app_id.clone(),
                                &role.hty_role_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_app_role = AppRole {
                                        the_id: uuid(),
                                        app_id: app_id.clone(),
                                        role_id: role.hty_role_id.clone().unwrap(),
                                    };
                                    AppRole::create(&in_app_role, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                    if role.labels.is_some() {
                        for label in role.labels.clone().unwrap() {
                            match RoleLabel::verify_exist_by_role_id_and_label_id(
                                &role.hty_role_id.clone().unwrap(),
                                &label.hty_label_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_role_label = RoleLabel {
                                        the_id: uuid(),
                                        role_id: role.hty_role_id.clone().unwrap(),
                                        label_id: label.hty_label_id.clone().unwrap(),
                                    };
                                    RoleLabel::create(&in_role_label, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                    if role.actions.is_some() {
                        for action in role.actions.clone().unwrap() {
                            match RoleAction::verify_exist_by_role_id_and_action_id(
                                &role.hty_role_id.clone().unwrap(),
                                &action.hty_action_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_role_action = RoleAction {
                                        the_id: uuid(),
                                        role_id: role.hty_role_id.clone().unwrap(),
                                        action_id: action.hty_action_id.clone().unwrap(),
                                    };
                                    RoleAction::create(&in_role_action, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid hty role id exits".into()),
                    }));
                }
            }
            Ok(role.clone())
        };
        return match exec_read_write_task(
            Box::new(task_update_role),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            _ => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("create_or_update_roles -> hty role updated error ".into()),
            })),
        };
    }
    Ok(role.clone())
}


async fn create_or_update_actions(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(hty_action): Json<ReqHtyAction>,
) -> Json<HtyResponse<ReqHtyAction>> {
    debug!("create_or_update_actions -> starts");
    match raw_create_or_update_actions(&hty_action, db_pool) {
        Ok(res) => {
            debug!(
                "create_or_update_actions -> success to create/update actions, action: {:?}!",
                res
            );
            wrap_json_ok_resp(res)
        }
        Err(e) => {
            error!(
                "create_or_update_actions -> failed to create/update actions, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_actions(
    hty_action: &ReqHtyAction,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyAction> {
    let action = hty_action.clone();
    if action.labels.is_some() {
        for label in action.labels.clone().unwrap() {
            match HtyLabel::verify_exist_by_id(
                &label.hty_label_id.unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid label id exits".into()),
                    }));
                }
            }
        }
    }
    if action.roles.is_some() {
        for role in action.roles.clone().unwrap() {
            match HtyRole::verify_exist_by_id(
                &role.hty_role_id.unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid role id exits".into()),
                    }));
                }
            }
        }
    }
    if action.action_name.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("action name can not be none".into()),
        }));
    }
    if action.action_status.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("action status can not be none".into()),
        }));
    }

    let params = HashMap::new();
    // action_id is none, then create new action
    if action.hty_action_id.is_none() {
        let task_create_action = move |_in_params: Option<HashMap<String, ReqHtyAction>>,
                                       conn: &mut PgConnection|
                                       -> anyhow::Result<ReqHtyAction> {
            let mut res_action = action.clone();
            let in_action = HtyAction {
                hty_action_id: uuid(),
                action_name: action.action_name.clone().unwrap(),
                action_desc: action.action_desc.clone(),
                action_status: action.action_status.clone().unwrap(),
            };
            let created_action = HtyAction::create(&in_action, conn)?;
            res_action.hty_action_id = Some(created_action.hty_action_id.clone());
            if action.labels.is_some() {
                for label in action.labels.clone().unwrap() {
                    let in_label = ActionLabel {
                        the_id: uuid(),
                        action_id: created_action.hty_action_id.clone(),
                        label_id: label.hty_label_id.clone().unwrap(),
                    };
                    ActionLabel::create(&in_label, conn)?;
                }
            }
            if action.roles.is_some() {
                for role in action.roles.clone().unwrap() {
                    let in_action = RoleAction {
                        the_id: uuid(),
                        role_id: role.hty_role_id.clone().unwrap(),
                        action_id: in_action.hty_action_id.clone(),
                    };
                    RoleAction::create(&in_action, conn)?;
                }
            }
            return Ok(res_action.clone());
        };
        return match exec_read_write_task(
            Box::new(task_create_action),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            _ => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("create_or_update_actions -> hty action updated error ".into()),
            })),
        };
    }

    // action_id is some, then update existing role
    if action.hty_action_id.is_some() {
        let task_update_action = move |_in_params: Option<HashMap<String, ReqHtyAction>>,
                                       conn: &mut PgConnection|
                                       -> anyhow::Result<ReqHtyAction> {
            RoleAction::delete_by_action_id(&action.hty_action_id.clone().unwrap(), conn)?;
            ActionLabel::delete_by_action_id(&action.hty_action_id.clone().unwrap(), conn)?;
            match HtyAction::verify_exist_by_id(&action.hty_action_id.clone().unwrap(), conn) {
                Ok(true) => {
                    let update_action = HtyAction {
                        hty_action_id: action.hty_action_id.clone().unwrap(),
                        action_name: action.action_name.clone().unwrap(),
                        action_desc: action.clone().action_desc,
                        action_status: action.action_status.clone().unwrap(),
                    };
                    HtyAction::update(&update_action, conn)?;
                    if action.labels.is_some() {
                        for label in action.labels.clone().unwrap() {
                            match ActionLabel::verify_exist_by_action_id_and_label_id(
                                &action.hty_action_id.clone().unwrap(),
                                &label.hty_label_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_action_label = ActionLabel {
                                        the_id: uuid(),
                                        action_id: action.hty_action_id.clone().unwrap(),
                                        label_id: label.hty_label_id.clone().unwrap(),
                                    };
                                    ActionLabel::create(&in_action_label, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                    if action.roles.is_some() {
                        for role in action.roles.clone().unwrap() {
                            match RoleAction::verify_exist_by_role_id_and_action_id(
                                &role.hty_role_id.clone().unwrap(),
                                &action.hty_action_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_role_action = RoleAction {
                                        the_id: uuid(),
                                        role_id: role.hty_role_id.clone().unwrap(),
                                        action_id: action.hty_action_id.clone().unwrap(),
                                    };
                                    RoleAction::create(&in_role_action, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid hty role id exits".into()),
                    }));
                }
            }
            Ok(action.clone())
        };
        return match exec_read_write_task(
            Box::new(task_update_action),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            _ => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("hty action updated error ".into()),
            })),
        };
    }

    Ok(action.clone())
}

async fn create_or_update_labels(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(hty_label): Json<ReqHtyLabel>,
) -> Json<HtyResponse<ReqHtyLabel>> {
    debug!("create_or_update_labels -> starts");
    match raw_create_or_update_labels(&hty_label, db_pool) {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!(
                "create_or_update_apps_with_labels -> failed to create/update labels, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_create_or_update_labels(
    hty_label: &ReqHtyLabel,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyLabel> {
    let label = hty_label.clone();
    if label.roles.is_some() {
        for role in label.roles.clone().unwrap() {
            match HtyRole::verify_exist_by_id(
                &role.hty_role_id.clone().unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("create_or_update_labels -> invalid role id exits".into()),
                    }));
                }
            }
        }
    }
    if label.actions.is_some() {
        for action in label.actions.clone().unwrap() {
            match HtyAction::verify_exist_by_id(
                &action.hty_action_id.clone().unwrap(),
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            ) {
                Ok(true) => continue,
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some("invalid role id exits".into()),
                    }));
                }
            }
        }
    }
    if label.label_name.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("label name can not be none".into()),
        }));
    }
    if label.label_status.is_none() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("label status can not be none".into()),
        }));
    }

    let params = HashMap::new();
    // label_id is none, then create new label
    if label.hty_label_id.is_none() {
        let task_create_label = move |_in_params: Option<HashMap<String, ReqHtyLabel>>,
                                      conn: &mut PgConnection|
                                      -> anyhow::Result<ReqHtyLabel> {
            let mut res_label = label.clone();
            let in_label = HtyLabel {
                hty_label_id: uuid(),
                label_name: label.label_name.clone().unwrap(),
                label_desc: label.label_desc.clone(),
                label_status: label.label_status.clone().unwrap(),
                style: label.style.clone(),
            };
            let created_label = HtyLabel::create(&in_label, conn)?;
            res_label.hty_label_id = Some(created_label.hty_label_id.clone());
            if label.actions.is_some() {
                for action in label.actions.clone().unwrap() {
                    let in_action_label = ActionLabel {
                        the_id: uuid(),
                        action_id: action.hty_action_id.clone().unwrap(),
                        label_id: created_label.hty_label_id.clone(),
                    };
                    ActionLabel::create(&in_action_label, conn)?;
                }
            }
            if label.roles.is_some() {
                for role in label.roles.clone().unwrap() {
                    let in_role_label = RoleLabel {
                        the_id: uuid(),
                        role_id: role.hty_role_id.clone().unwrap(),
                        label_id: created_label.hty_label_id.clone(),
                    };
                    RoleLabel::create(&in_role_label, conn)?;
                }
            }
            return Ok(res_label.clone());
        };

        return match exec_read_write_task(
            Box::new(task_create_label),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some(
                    "create_or_update_labels -> hty label create error ".to_string()
                        + &e.to_string()
                ),
            })),
        };
    }

    // label_id is some, then update existing label
    if label.hty_label_id.is_some() {
        let task_update_label = move |_in_params: Option<HashMap<String, ReqHtyLabel>>,
                                      conn: &mut PgConnection|
                                      -> anyhow::Result<ReqHtyLabel> {
            RoleLabel::delete_by_label_id(&label.hty_label_id.clone().unwrap(), conn)?;
            ActionLabel::delete_by_label_id(&label.hty_label_id.clone().unwrap(), conn)?;
            match HtyLabel::verify_exist_by_id(&label.hty_label_id.clone().unwrap(), conn) {
                Ok(true) => {
                    let update_label = HtyLabel {
                        hty_label_id: label.hty_label_id.clone().unwrap(),
                        label_name: label.label_name.clone().unwrap(),
                        label_desc: label.clone().label_desc,
                        label_status: label.label_status.clone().unwrap(),
                        style: label.style.clone(),
                    };
                    HtyLabel::update(&update_label, conn)?;
                    if label.actions.is_some() {
                        for action in label.actions.clone().unwrap() {
                            match ActionLabel::verify_exist_by_action_id_and_label_id(
                                &action.hty_action_id.clone().unwrap(),
                                &label.hty_label_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_action_label = ActionLabel {
                                        the_id: uuid(),
                                        action_id: action.hty_action_id.clone().unwrap(),
                                        label_id: label.hty_label_id.clone().unwrap(),
                                    };
                                    ActionLabel::create(&in_action_label, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                    if label.roles.is_some() {
                        for role in label.roles.clone().unwrap() {
                            match RoleLabel::verify_exist_by_role_id_and_label_id(
                                &role.hty_role_id.clone().unwrap(),
                                &label.hty_label_id.clone().unwrap(),
                                conn,
                            ) {
                                Ok(true) => continue,
                                Ok(false) => {
                                    let in_role_label = RoleLabel {
                                        the_id: uuid(),
                                        role_id: role.hty_role_id.clone().unwrap(),
                                        label_id: label.hty_label_id.clone().unwrap(),
                                    };
                                    RoleLabel::create(&in_role_label, conn)?;
                                }
                                Err(e) => {
                                    return Err(anyhow!(HtyErr {
                                        code: HtyErrCode::NullErr,
                                        reason: Some(e.to_string()),
                                    }));
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some(
                            "create_or_update_labels -> invalid hty label id exits".into()
                        ),
                    }));
                }
            }
            Ok(label.clone())
        };
        return match exec_read_write_task(
            Box::new(task_update_label),
            Some(params),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some(
                    "create_or_update_labels -> hty label updated error ".to_string()
                        + &e.to_string()
                ),
            })),
        };
    }

    Ok(label.clone())
}

async fn delete_user_group(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<()>> {
    debug!("delete_user_group -> starts");
    match raw_delete_user_group(&id, db_pool) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!(
                "delete_user_group -> failed to delete user_group id {}, e: {}",
                id, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_delete_user_group(id: &String, db_pool: Arc<DbState>) -> anyhow::Result<()> {
    let mut hty_user_group =
        HtyUserGroup::find_by_id(&id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    hty_user_group.is_delete = true;
    let _ = HtyUserGroup::update(
        &hty_user_group,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    Ok(())
}

async fn delete_app_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<HtyApp>> {
    debug!("delete_app_by_id -> starts");
    match raw_delete_app_by_id(&id, db_pool) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!(
                "delete_app_by_id -> failed to delete app by id {}, e: {}",
                id, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_delete_app_by_id(id: &String, db_pool: Arc<DbState>) -> anyhow::Result<HtyApp> {
    let mut hty_app = HtyApp::find_by_id(&id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    hty_app.app_status = APP_STATUS_DELETED.to_string();
    let res = HtyApp::update(&hty_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    Ok(res)
}

pub async fn find_all_valid_teachers(
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqHtyUserWithInfos>>> {
    debug!("find_all_valid_teachers -> starts");
    let role = String::from("TEACHER");
    match raw_find_users_with_info_by_role(&role, host, extract_conn(conn).deref_mut()) {
        Ok(in_users) => {
            let mut resp = Vec::new();
            for in_user in in_users {
                if in_user.enabled.is_some() {
                    let is_enabled = in_user.enabled.clone().unwrap();
                    if is_enabled && in_user.infos.clone().unwrap().get(0).unwrap().is_registered {
                        resp.push(in_user.clone());
                    }
                }
            }
            debug!(
                "find_all_valid_teachers -> success to find all valid teachers: {:?}!",
                resp
            );
            wrap_json_ok_resp(resp)
        }
        Err(e) => {
            error!(
                "find_all_valid_teachers -> failed to find all teachers. e: {}",
                e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

pub async fn find_users_in_app(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
    hty_ids: Json<Vec<String>>,
) -> Json<HtyResponse<Vec<ReqHtyUserWithInfos>>> {
    debug!("find_users_in_app -> starts");
    let ids = hty_ids.0;

    debug!("find_users_in_app -> ids: {:?}", &ids);

    match raw_find_users_in_app(ids, &host, db_pool) {
        Ok(ok) => {
            debug!("find_users_in_app -> success to find bulk user: {:?}", ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!("find_users_in_app -> failed to find bulk user. e: {}", e);
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

pub async fn find_hty_users_by_ids(
    Query(params): Query<HashMap<String, String>>,
    _sudoer: HtySudoerTokenHeader,
    _host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
    hty_ids: Json<Vec<String>>,
) -> Json<HtyResponse<(Vec<ReqHtyUser>, i64, i64)>> {
    debug!("find_hty_users_by_ids -> starts");
    let ids = hty_ids.0;
    let (page, page_size) = get_page_and_page_size(&params);

    debug!("find_hty_users_by_ids -> ids: {:?}", &ids);

    match raw_find_hty_users_by_ids(ids, &page, &page_size, db_pool) {
        Ok(ok) => {
            debug!("raw_find_hty_users_by_ids -> success to find bulk user: {:?}", ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!("raw_find_hty_users_by_ids -> failed to find bulk user. e: {}", e);
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_find_hty_users_by_ids(
    hty_ids: Vec<String>,
    page: &Option<i64>,
    page_size: &Option<i64>,
    db_pool: Arc<DbState>,
) -> anyhow::Result<(Vec<ReqHtyUser>, i64, i64)> {
    let (found_users, total_page, total) = HtyUser::find_all_hty_users_by_hty_ids(&hty_ids, page, page_size, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    let to_req_users = HtyUser::to_req_users(&found_users);

    let res = (to_req_users, total_page, total);

    debug!("raw_find_hty_users_by_ids -> res: {:?}", res);

    Ok(res)
}

fn raw_find_users_in_app(
    hty_ids: Vec<String>,
    host: &HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Vec<ReqHtyUserWithInfos>> {
    let mut res = vec![];
    for hty_id in hty_ids {
        let cloned_hty_id = hty_id.clone();
        debug!(
            "raw_find_users_in_app -> hty_id: {:?} / host {:?}",
            &hty_id,
            host,
        );
        match raw_find_user_with_info_by_id_and_host(
            &hty_id,
            host,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        ) {
            Ok(item) => {
                debug!("raw_find_user_with_info_by_id_and_host -> FOUND hty_id: {:?} / {:?}", cloned_hty_id, item);
                res.push(item);
            }
            Err(err) => {
                warn!("raw_find_user_with_info_by_id_and_host -> hty_id: {:?} / {:?}", cloned_hty_id, err);
            }
        };
    };

    debug!("raw_find_users_in_app -> res: {:?}", res);

    Ok(res)
}

pub async fn find_users_with_info_by_role(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    Path(role): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqHtyUserWithInfos>>> {
    debug!("find_users_with_info_by_role -> starts");
    match raw_find_users_with_info_by_role(&role, host, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("find_users_with_info_by_role -> success to find user with info by role: {}, user with info: {:?}!", role, ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "find_users_with_info_by_role -> failed to find user with info by role: {}. e: {}",
                role, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_find_users_with_info_by_role(
    role: &String,
    host: HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<ReqHtyUserWithInfos>> {
    let domain = (*host).clone();
    let app_id = HtyApp::find_by_domain(&domain, conn)?;
    let role = HtyRole::find_by_key(role, conn)?;
    let user_infos = role.find_linked_user_app_info(conn)?;
    let out_user_infos: Vec<UserAppInfo> = user_infos
        .into_iter()
        .filter(|item| item.clone().app_id.unwrap() == app_id.app_id)
        .collect();
    let out_req_user_infos: Vec<ReqUserAppInfo> =
        out_user_infos.iter().map(|item| item.to_req()).collect();
    let res: Vec<ReqHtyUserWithInfos> = out_req_user_infos
        .iter()
        .map(|item| {
            let user = HtyUser::find_by_hty_id(&item.clone().hty_id.unwrap(), conn).unwrap();
            let mut infos = Vec::new();
            infos.push(item.clone());
            let out = ReqHtyUserWithInfos {
                hty_id: Some(user.hty_id.clone()),
                union_id: user.union_id.clone(),
                enabled: Some(user.enabled),
                created_at: user.created_at.clone(),
                real_name: user.real_name.clone(),
                sex: user.sex.clone(),
                mobile: user.mobile.clone(),
                infos: Some(infos),
                info_roles: None,
                settings: user.settings.clone(),
            };
            out
        })
        .collect();
    Ok(res)
}

pub async fn find_user_with_info_by_id_and_host(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyUserWithInfos>> {
    debug!("find_user_with_info_by_id_and_host -> starts");
    match raw_find_user_with_info_by_id_and_host(&id, &host, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("find_user_with_info_by_id_and_host -> success to find user with info by id and host: {}, user with info: {:?}!", id, ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!("find_user_with_info_by_id_and_host -> failed to find user with info by id and host: {}. e: {}", id, e);
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_find_user_with_info_by_id_and_host(
    id_hty: &String,
    host: &HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyUserWithInfos> {
    let in_app = get_app_from_host((*host).clone(), conn)?;

    debug!("raw_find_user_with_info_by_id_and_host -> hty_id: {:?} / in_app: {:?}", id_hty, in_app);

    let user = HtyUser::find_by_hty_id(id_hty.as_str(), conn)?;
    debug!("raw_find_user_with_info_by_id_and_host -> user: {:?}", user);

    let user_app_info = UserAppInfo::find_by_hty_id_and_app_id(&id_hty, &in_app.app_id, conn)?;
    debug!("raw_find_user_with_info_by_id_and_host -> user_app_info: {:?}", user_app_info);


    let roles = user_app_info.find_linked_roles(conn)?;
    debug!("raw_find_user_with_info_by_id_and_host -> roles: {:?}", roles);

    let req_roles: Vec<ReqHtyRole> = roles
        .iter()
        .map(|role| ReqHtyRole {
            hty_role_id: Some(role.hty_role_id.clone()),
            user_app_info_id: None,
            app_ids: None,
            role_key: Some(role.role_key.clone()),
            role_desc: role.role_desc.clone(),
            role_status: Some(role.role_status.clone()),
            labels: None,
            actions: None,
            style: role.style.clone(),
            role_name: role.role_name.clone(),
        })
        .collect();


    debug!("raw_find_user_with_info_by_id_and_host -> req_roles: {:?}", req_roles);

    let res_req_user_app_info = ReqUserAppInfo {
        id: Some(user_app_info.id.clone()),
        app_id: user_app_info.app_id.clone(),
        hty_id: Some(user_app_info.hty_id.clone()),
        openid: user_app_info.openid.clone(),
        is_registered: user_app_info.is_registered,
        username: user_app_info.username.clone(),
        password: user_app_info.password.clone(),
        roles: Some(req_roles),
        meta: user_app_info.meta.clone(),
        created_at: user_app_info.created_at.clone(),
        teacher_info: user_app_info.teacher_info.clone(),
        student_info: user_app_info.student_info.clone(),
        reject_reason: user_app_info.reject_reason.clone(),
        needs_refresh: user_app_info.needs_refresh.clone(),
        unread_tongzhi_count: None,
        avatar_url: user_app_info.avatar_url.clone(),
    };


    debug!("raw_find_user_with_info_by_id_and_host -> res_req_user_app_info: {:?}", res_req_user_app_info);

    let out_user = HtyUser::to_req_user(&user);
    let mut out_infos = Vec::new();
    out_infos.push(res_req_user_app_info);

    let resp = ReqHtyUserWithInfos {
        hty_id: out_user.hty_id.clone(),
        union_id: out_user.union_id.clone(),
        enabled: out_user.enabled.clone(),
        created_at: out_user.created_at.clone(),
        real_name: out_user.real_name.clone(),
        sex: out_user.sex.clone(),
        mobile: out_user.mobile.clone(),
        infos: Some(out_infos),
        info_roles: None,
        settings: out_user.settings.clone(),
    };

    debug!("raw_find_user_with_info_by_id_and_host: resp -> {:?}", &resp);
    Ok(resp)
}

pub async fn find_hty_user_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyUser>> {
    debug!("find_hty_user_by_id -> starts");
    match raw_find_hty_user_by_id(&id, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("find_hty_user_by_id -> success to find user by id: {:?}, {:?}", id, ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!("find_hty_user_by_id -> failed to find user by id: {:?}. e: {:?}", id, e);
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_find_hty_user_by_id(
    id_hty: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyUser> {
    debug!("raw_find_hty_user_by_id -> hty_id: {:?}", id_hty);

    let user = HtyUser::find_by_hty_id(id_hty.as_str(), conn)?;
    debug!("raw_find_hty_user_by_id -> user: {:?}", user);

    let resp = ReqHtyUser {
        hty_id: Some(user.hty_id),
        union_id: user.union_id,
        enabled: Some(user.enabled),
        created_at: user.created_at,
        real_name: user.real_name,
        sex: user.sex,
        mobile: user.mobile,
        settings: user.settings,
    };

    debug!("raw_find_hty_user_by_id: resp -> {:?}", &resp);
    Ok(resp)
}

pub async fn find_user_with_info_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyUserWithInfos>> {
    debug!("find_user_with_info_by_id -> starts");
    match raw_find_user_with_info_by_id(&id, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("find_user_with_info_by_id -> success to find user with info by id: {}, user with info: {:?}!", id, ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "find_user_with_info_by_id -> failed to find user with info by id: {}. e: {}",
                id, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

fn raw_find_user_with_info_by_id(
    id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyUserWithInfos> {
    let user = HtyUser::find_by_hty_id(id.as_str(), conn)?;
    let vs = UserAppInfo::find_all_user_infos_by_hty_id(id.as_str(), conn)?;

    let req_user_app_infos: anyhow::Result<Vec<ReqUserAppInfo>> = vs
        .iter()
        .map(|user_info| {
            // let roles = user_info.roles_by_id(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
            let roles = user_info.find_linked_roles(conn)?;
            if !roles.is_empty() {
                let unwarp_roles = roles.clone();
                let req_roles: Vec<ReqHtyRole> = unwarp_roles
                    .into_iter()
                    .map(|role| ReqHtyRole {
                        hty_role_id: Some(role.hty_role_id.clone()),
                        user_app_info_id: None,
                        app_ids: None,
                        role_key: Some(role.role_key.clone()),
                        role_desc: role.role_desc.clone(),
                        role_status: Some(role.role_status.clone()),
                        labels: None,
                        actions: None,
                        style: role.style.clone(),
                        role_name: role.role_name.clone(),
                    })
                    .collect();
                return Ok(ReqUserAppInfo {
                    id: Some(user_info.id.clone()),
                    app_id: user_info.app_id.clone(),
                    hty_id: Some(user_info.hty_id.clone()),
                    openid: user_info.openid.clone(),
                    is_registered: user_info.is_registered,
                    username: user_info.username.clone(),
                    password: user_info.password.clone(),
                    roles: Some(req_roles),
                    meta: user_info.meta.clone(),
                    created_at: user_info.created_at.clone(),
                    teacher_info: user_info.teacher_info.clone(),
                    student_info: user_info.student_info.clone(),
                    reject_reason: user_info.reject_reason.clone(),
                    needs_refresh: user_info.needs_refresh.clone(),
                    unread_tongzhi_count: None,
                    avatar_url: user_info.avatar_url.clone(),
                });
            } else {
                return Ok(ReqUserAppInfo {
                    id: Some(user_info.id.clone()),
                    app_id: user_info.app_id.clone(),
                    hty_id: Some(user_info.hty_id.clone()),
                    openid: user_info.openid.clone(),
                    is_registered: user_info.is_registered,
                    username: user_info.username.clone(),
                    password: user_info.password.clone(),
                    roles: None,
                    meta: user_info.meta.clone(),
                    created_at: Some(current_local_datetime()),
                    teacher_info: user_info.teacher_info.clone(),
                    student_info: user_info.student_info.clone(),
                    reject_reason: user_info.reject_reason.clone(),
                    needs_refresh: user_info.needs_refresh.clone(),
                    unread_tongzhi_count: None,
                    avatar_url: user_info.avatar_url.clone(),
                });
            }
        })
        .collect();

    let out_user = HtyUser::to_req_user(&user);
    let out = ReqHtyUserWithInfos {
        hty_id: out_user.hty_id.clone(),
        union_id: out_user.union_id.clone(),
        enabled: out_user.enabled.clone(),
        created_at: out_user.created_at.clone(),
        real_name: out_user.real_name.clone(),
        sex: out_user.sex.clone(),
        mobile: out_user.mobile.clone(),
        infos: Some(req_user_app_infos?),
        info_roles: None,
        settings: out_user.settings.clone(),
    };
    Ok(out)
}

pub async fn find_user_with_info_by_token(
    host: HtyHostHeader,
    auth: AuthorizationHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<ReqHtyUserWithInfos>> {
    debug!("find_user_with_info_by_token -> starts");
    match raw_find_user_with_info_by_token(host, auth, db_conn, db_pool).await {
        Ok(ok) => {
            debug!(
                "find_user_with_info_by_token -> success find users: {:?}",
                ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "find_user_with_info_by_token -> fail to find user, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_find_user_with_info_by_token(
    host: HtyHostHeader,
    auth: AuthorizationHeader,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<ReqHtyUserWithInfos> {
    let _ = verify_jwt(&(*auth).clone())?;

    let token = jwt_decode_token(&(*auth).clone())?;

    debug(format!("raw_find_user_with_info_by_token -> token: {:?}", token).as_str());

    let domain = (*host).clone();

    debug(format!("raw_find_user_with_info_by_token -> domain: {:?}", domain).as_str());

    let in_app =
        HtyApp::find_by_domain(&domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    debug(format!("raw_find_user_with_info_by_token -> app_id: {:?}", in_app).as_str());

    let in_user = HtyUser::find_by_hty_id(
        &token.hty_id.clone().unwrap()[..],
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    debug(format!("raw_find_user_with_info_by_token -> user: {:?}", in_user).as_str());

    let info = UserAppInfo::find_by_hty_id_and_app_id(
        &token.hty_id.clone().unwrap(),
        &in_app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    debug(format!("raw_find_user_with_info_by_token -> info: {:?}", info).as_str());

    let linked_roles =
        info.find_linked_roles(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let mut req_roles = Vec::new();

    for in_role in linked_roles {
        let mut out_role = ReqHtyRole {
            hty_role_id: Some(in_role.hty_role_id.clone()),
            user_app_info_id: None,
            app_ids: None,
            role_key: Some(in_role.role_key.clone()),
            role_desc: in_role.role_desc.clone(),
            role_status: Some(in_role.role_status.clone()),
            labels: None,
            actions: None,
            style: in_role.style.clone(),
            role_name: in_role.role_name.clone(),
        };

        let labels = HtyLabel::find_all_by_role_id(
            &out_role.hty_role_id.clone().unwrap(),
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        out_role.labels = Some(HtyLabel::to_req_labels(&labels));

        req_roles.push(out_role);
    }

    let mut out_user_info = UserAppInfo::to_req(&info);
    out_user_info.roles = Some(req_roles);

    let out_user = HtyUser::to_req_user(&in_user);
    let mut infos = Vec::new();
    infos.push(out_user_info);

    let req_hty_user_with_infos = ReqHtyUserWithInfos {
        hty_id: out_user.hty_id.clone(),
        union_id: out_user.union_id.clone(),
        enabled: out_user.enabled.clone(),
        created_at: out_user.created_at.clone(),
        real_name: out_user.real_name.clone(),
        sex: out_user.sex.clone(),
        mobile: out_user.mobile.clone(),
        infos: Some(infos),
        info_roles: None,
        settings: out_user.settings.clone(),
    };

    debug(format!("raw_find_user_with_info_by_token -> req_hty_user_with_infos: {:?}", req_hty_user_with_infos).as_str());

    // 这里获取不到小程序`openid`
    // task::spawn(async move {
    //     let _ = call_refresh_openid(&out_user.hty_id.clone().unwrap(), &info.id.clone()).await;
    // });

    task::spawn(async move {
        let _ = post_login(
            &in_user,
            &in_app.clone(),
            extract_conn(fetch_db_conn(&db_pool).unwrap()).deref_mut(),
        )
            .await; // todo: unwrap should be fixed.
    });

    Ok(req_hty_user_with_infos)
}

pub async fn sudo2(auth: AuthorizationHeader, host: HtyHostHeader, conn: db::DbConn, Path(to_user_id): Path<String>) -> Json<HtyResponse<String>> {
    debug!("sudo2 -> starts");
    match raw_sudo2(auth, host, to_user_id, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("sudo2 -> success sudo: {:?}", ok);
            wrap_json_ok_resp(ok)
        }
        Err(err) => {
            error!("sudo2 -> failed to sudo, e: {:?}", err);
            wrap_json_anyhow_err(err)
        }
    }
}

fn raw_sudo2(auth: AuthorizationHeader, host: HtyHostHeader, to_user_id: String, conn: &mut PgConnection) -> anyhow::Result<String> {
    let _ = verify_jwt(&(*auth).clone())?;

    debug!("raw_sudo2 -> auth: {:?}", auth);

    let the_auth = auth.clone();

    let user_tags = get_all_db_tags_of_the_user(auth, host, conn)?;
    debug!("raw_sudo2 -> user_tags: {:?}", user_tags);

    let id_current_user = jwt_decode_token(&(*the_auth).clone())?.hty_id.unwrap();

    let user_app_info = UserAppInfo::find_by_id(&to_user_id, conn)?;
    let id_to_user = user_app_info.hty_id.clone();

    if id_current_user == id_to_user || user_tags.iter().any(|item| item.tag_name == "SYS_CAN_SUDO") {
        let resp_token = HtyToken {
            token_id: uuid(),
            hty_id: Some(user_app_info.hty_id.clone()),
            app_id: None,
            ts: current_local_datetime(),
            roles: user_app_info.req_roles_by_id(conn)?,
            tags: None,
        };
        save_token_with_exp_days(&resp_token, get_token_expiration_days())?;
        Ok(jwt_encode_token(resp_token)?)
    } else {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("no SYS_CAN_SUDO tag".into()),
        }));
    }
}

pub async fn sudo(auth: AuthorizationHeader, conn: db::DbConn) -> Json<HtyResponse<String>> {
    debug!("sudo -> starts");
    match raw_sudo(auth, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("sudo -> success sudo: {:?}", ok);
            wrap_json_ok_resp(ok)
        }
        Err(err) => {
            error!("sudo -> failed to sudo, e: {:?}", err);
            wrap_json_anyhow_err(err)
        }
    }
}

// todo：ADD `SYS_SUDO` TAG TO VERIFY.
fn raw_sudo(auth: AuthorizationHeader, conn: &mut PgConnection) -> anyhow::Result<String> {
    let jwt_token = &(*auth).clone();

    debug!("raw_sudo -> in_token: {:?}", jwt_token);

    match verify_jwt(&jwt_token) {
        Ok(()) => {
            debug(format!("raw_sudo -> verify jwt token okay! -> {:?}", jwt_token).as_str());

            // todo: Currently sudoer token = root token
            // In the future we can switch to other ROLE.
            let root_app_id = HtyApp::find_by_domain(&"root".to_string(), conn)?.app_id;
            debug!(
                "raw_sudo -> find_by_domain -> root_app_id: {:?}",
                root_app_id
            );

            let root_user_info =
                UserAppInfo::find_by_username_and_app_id(&"root".to_string(), &root_app_id, conn)?;

            let resp_token = HtyToken {
                token_id: uuid(),
                hty_id: Some(root_user_info.hty_id.clone()),
                app_id: None,
                ts: current_local_datetime(),
                roles: root_user_info.req_roles_by_id(conn)?,
                tags: None,
            };

            //Save sudoer token
            save_token_with_exp_days(&resp_token, get_token_expiration_days())?;

            Ok(jwt_encode_token(resp_token)?)
        }
        Err(e) => {
            debug(
                format!(
                    "raw_sudo -> raw verify jwt token error! -> TOKEN: {:?} / ERR: {:?}",
                    jwt_token, e
                )
                    .as_str(),
            );
            Err(anyhow!(e))
        }
    }
}

pub async fn login_with_password(
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req_login): Json<ReqLogin>,
) -> impl IntoResponse {
    let app_domain = (*host).clone();
    debug!("login_with_password -> starts {:?} / {:?}", req_login, host);

    match raw_login_with_password(&app_domain, &req_login, db_pool) {
        Ok(ok) => {
            debug!(
                "login_with_password -> login success {:?} / {:?}",
                req_login, app_domain
            );
            (StatusCode::OK, wrap_json_ok_resp(ok))
        }
        Err(err) => {
            error!(
                "login_with_password -> failed to login, e: {:?} / info: {:?}",
                err, req_login
            );
            (StatusCode::UNAUTHORIZED, wrap_json_anyhow_err(err))
        }
    }
}

fn raw_login_with_password(
    app_domain: &String,
    in_req_login: &ReqLogin,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    let app = HtyApp::find_by_domain(
        app_domain,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let req_login = in_req_login.clone();
    if req_login.username.is_none() || req_login.password.is_none() {
        error!("raw_login_with_password -> username or password null");
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("username or password error".into()),
        }));
    }

    let info = UserAppInfo::find_by_username_and_app_id(
        &req_login.username.clone().unwrap(),
        &app.app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    let resp_token = HtyToken {
        token_id: uuid(),
        hty_id: Some(info.hty_id.clone()),
        app_id: None,
        ts: current_local_datetime(),
        roles: info.req_roles_by_id(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?,
        tags: None,
    };

    return if info.password == req_login.password {
        save_token_with_exp_days(&resp_token, get_token_expiration_days())?;

        Ok(jwt_encode_token(resp_token)?)
    } else {
        Err(anyhow!(HtyErr {
            code: HtyErrCode::AuthenticationFailed,
            reason: Some("user password authentication failed".into()),
        }))
    };
}

pub async fn wx_qr_login(State(db_pool): State<Arc<DbState>>, host: HtyHostHeader, Json(req_code): Json<ReqQRCode>) -> impl IntoResponse {
    debug!("wx_qr_login -> starts");

    let app_domain = (*host).clone();
    let code = req_code.code.unwrap();

    debug!("wx_qr_login -> app_domain {:?} / code {:?}", &app_domain, &code);

    match raw_wx_qr_login(code, app_domain, db_pool).await {
        Ok(ok) => {
            debug!("raw_wx_qr_login -> login success");
            (StatusCode::OK, wrap_json_ok_resp(ok))
        }
        Err(e) => {
            error!("raw_wx_qr_login -> failed to login, e: {:?}", e);
            (StatusCode::UNAUTHORIZED, wrap_json_anyhow_err(e))
        }
    }
}

async fn raw_wx_qr_login(code: String, app_domain: String, db_pool: Arc<DbState>) -> anyhow::Result<String> {
    debug!("raw_wx_qr_login -> domain: {:?} / code: {:?}", &app_domain, &code);

    let app = HtyApp::find_by_domain(&app_domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let wx_secret = app.wx_secret.clone().unwrap();

    debug!("raw_wx_qr_login -> domain: {:?} / app: {:?} / secret: {:?} / code: {:?}", &app_domain, &app, &wx_secret, &code);

    let union_id = get_union_id_by_auth_code(app.wx_id.clone().unwrap(), wx_secret, code).await?;

    debug!("raw_wx_qr_login -> union_id: {:?}", &union_id);

    let user = HtyUser::find_by_union_id(&union_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    // 第一次登录则不存在本app对应的user_app_info
    let user_is_exist_in_this_app = UserAppInfo::verify_exist_by_app_id_and_hty_id(&user.hty_id, &app.app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    debug!("raw_wx_qr_login -> user_is_exist_in_this_app: {:?}", &user_is_exist_in_this_app);

    let this_app_user_info;

    if !user_is_exist_in_this_app {
        let create_user_info = UserAppInfo {
            hty_id: user.hty_id.clone(),
            app_id: Some(app.app_id.clone()),
            openid: None,
            is_registered: false,
            id: uuid(),
            username: None,
            password: None,
            meta: None,
            created_at: Some(current_local_datetime()),
            teacher_info: None,
            student_info: None,
            reject_reason: None,
            needs_refresh: Some(false),
            avatar_url: None,
        };

        debug!("raw_wx_qr_login -> created_user_info: {:?}", &create_user_info);
        // 这里应该允许传入一个option，看看要不要自动创建user_app_info，还是返回为认证失败。
        this_app_user_info = UserAppInfo::create(&create_user_info, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    } else {
        this_app_user_info = UserAppInfo::find_by_hty_id_and_app_id(&user.hty_id, &app.app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

        if !this_app_user_info.is_registered {
            return Err(anyhow!("User is not registered in this app!"));
        }
    }
    debug!("raw_wx_qr_login -> this_app_user_info: {:?}", &this_app_user_info);

    let resp_token = HtyToken {
        token_id: uuid(),
        hty_id: Some(user.hty_id.clone()),
        app_id: Some(app.app_id.clone()),
        ts: current_local_datetime(),
        roles: this_app_user_info.req_roles_by_id(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?,
        tags: None,
    };

    debug!("raw_wx_qr_login -> resp_token: {:?}", &resp_token);

    save_token_with_exp_days(&resp_token, get_token_expiration_days())?;
    Ok(jwt_encode_token(resp_token)?)
}

async fn verify_user_enabled_and_registered_in_app(_sudoer: HtySudoerTokenHeader,
                                      _host: HtyHostHeader,
                                      Query(params): Query<HashMap<String, String>>,
                                      State(db_pool): State<Arc<DbState>>) -> Json<HtyResponse<bool>> {
    let in_app_domain = get_some_from_query_params::<String>("app_domain", &params);
    let in_hty_id = get_some_from_query_params::<String>("hty_id", &params);

    if in_app_domain.is_none() || in_hty_id.is_none() {
        return wrap_json_hty_err::<bool>(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some("app_domain or hty_id is none".into()),
        });
    }

    match raw_verify_user_enabled_and_registered_in_app(in_app_domain.unwrap(), in_hty_id.unwrap(), db_pool).await {
        Ok(result) => {
            debug!("verify_user_enabled_and_registered_in_app -> success: {:?}", result);
            wrap_json_ok_resp(result)
        }
        Err(e) => {
            error!("verify_user_enabled_and_registered_in_app -> failed, e: {:?}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

pub async fn raw_verify_user_enabled_and_registered_in_app(app_domain: String, hty_id: String, db_pool: Arc<DbState>) -> anyhow::Result<bool> {

    let in_user = HtyUser::find_by_hty_id(&hty_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    debug!("verify_user_enabled_and_registered_in_app -> in_user: {:?}", &in_user);

    let in_app = HtyApp::find_by_domain(&app_domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    let is_user_info_exists = UserAppInfo::verify_exist_by_app_id_and_hty_id(&hty_id, &in_app.app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    debug!("verify_user_enabled_and_registered_in_app -> user_is_exist_in_music_room_app: {:?}", &is_user_info_exists);

    if !is_user_info_exists {
        return Err(anyhow!("user info not exists!"));
    }

    let user_info = UserAppInfo::find_by_hty_id_and_app_id(&hty_id, &in_app.app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    debug!("verify_user_enabled_and_registered_in_app -> user_info: {:?}", &user_info);

    if in_user.enabled || user_info.is_registered {
        Ok(true)
    } else {
        Ok(false)
    }
}

// buddy: todo fix deprecated macro
#[debug_handler]
pub async fn login2_with_unionid(
    host: HtyHostHeader,
    union_id: UnionIdHeader,
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
) -> impl IntoResponse {
    debug!("login2_with_unionid -> starts");
    match raw_login2_with_unionid(host, union_id, db_conn, db_pool).await {
        Ok(ok) => {
            debug!("login2_with_unionid -> login success");
            (StatusCode::OK, wrap_json_ok_resp(ok))
        }
        Err(e) => {
            error!("raw_login2_with_unionid -> failed to login, e: {:?}", e);
            (StatusCode::UNAUTHORIZED, wrap_json_anyhow_err(e))
        }
    }
}

async fn raw_login2_with_unionid(
    host: HtyHostHeader,
    union: UnionIdHeader,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    raw_login2_with_unionid_tx(host, union, db_pool)
}

fn raw_login2_with_unionid_tx(
    host: HtyHostHeader,
    union: UnionIdHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    let union_id = (*union).clone();
    let from_app_domain = (*host).clone();

    debug(format!("login2_with_unionid -> {:?}", union_id.clone()).as_str());

    let no_use: HashMap<String, String> = HashMap::new();

    let task = move |_in_params: Option<HashMap<String, String>>,
                     conn: &mut PgConnection|
                     -> anyhow::Result<(String, HtyUser, HtyApp)> {
        let mut in_processing_union_id = LOGIN_UNION_ID_PREFIX.to_string().clone();
        in_processing_union_id.push_str(union_id.as_str());
        let in_processing_unionid_r = get_value_from_redis(&in_processing_union_id);

        match in_processing_unionid_r {
            Ok(_already_in_login_process) => {
                let msg =
                    format!("already in login process! union_id -> {:?}", union_id).to_string();
                debug!("{}", msg);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::ConflictErr,
                    reason: Some(msg),
                }))
            }
            Err(_not_in_login_process) => {
                // 这里在save之前还得检查一次，否则可能上面检查过了这里仍然可能存在二次登录的问题。
                let r2 = match get_value_from_redis(&in_processing_union_id) {
                    Ok(_still_already_logged_in) => {
                        let msg = format!("already in login process! union_id -> {:?}", union_id)
                            .to_string();
                        debug!("{}", msg);
                        Err(anyhow!(HtyErr {
                            code: HtyErrCode::ConflictErr,
                            reason: Some(msg),
                        }))
                    }
                    Err(_no_problem_now) => {
                        // 如果用户登录过程超过120秒，则认为此次登录已经超时了
                        let _ =
                            save_kv_to_redis_with_exp_secs(&in_processing_union_id, &union_id, 60);
                        Ok(())
                    }
                };

                if r2.is_err() {
                    return Err(anyhow!(r2.err().unwrap()));
                }

                let user_exist_r = match HtyUser::verify_exist_by_union_id(&union_id, conn) {
                    Ok(user_exist) => Ok(user_exist),
                    Err(err) => {
                        let _ = del_from_redis(&in_processing_union_id);
                        Err(err)
                    }
                };

                // 新用户没走`register()`流程，所以此时不存在`union_id`，所以直接返回了，不走下面流程。
                if user_exist_r.is_err() {
                    return Err(anyhow!("No authorized to login"));
                }

                let user_exist = user_exist_r?;

                if !user_exist {
                    let _ = del_from_redis(&in_processing_union_id);
                    return Err(anyhow!("No authorized to login"));
                }

                debug(
                    format!(
                        "login2_with_unionid / user_exist -> {:?}",
                        user_exist.clone()
                    )
                        .as_str(),
                );

                debug(
                    format!(
                        "login2_with_unionid / from_domain -> {:?}",
                        &from_app_domain
                    )
                        .as_str(),
                );

                let from_app = match HtyApp::find_by_domain(&from_app_domain, conn) {
                    Ok(app) => Some(app),
                    Err(_) => {
                        let _ = del_from_redis(&in_processing_union_id);
                        None
                    }
                };

                debug(format!("login2_with_unionid / from_app -> {:?}", &from_app).as_str());

                let ok_user = match HtyUser::find_by_union_id(&union_id, conn) {
                    Ok(in_user) => Ok(in_user),
                    Err(err) => {
                        let _ = del_from_redis(&in_processing_union_id);
                        Err(err)
                    }
                };

                // 上面已经DEL REDIS了
                if ok_user.is_err() {
                    return Err(anyhow!("Not authorized to login"));
                }

                let login_user = ok_user?;

                let token = match from_app.clone() {
                    Some(some_app) => HtyToken {
                        token_id: uuid(),
                        hty_id: Some(login_user.hty_id.clone()),
                        app_id: None,
                        ts: current_local_datetime(),
                        roles: login_user
                            .info(&some_app.app_id, conn)?
                            .req_roles_by_id(conn)?,
                        tags: None,
                    },
                    None => HtyToken {
                        token_id: uuid(),
                        hty_id: Some(login_user.hty_id.clone()),
                        app_id: None,
                        ts: current_local_datetime(),
                        roles: None,
                        tags: None,
                    },
                };

                save_token_with_exp_days(&token, get_token_expiration_days())?;
                debug!("entering post_login()");

                // 注册流程完成
                let _ = del_from_redis(&in_processing_union_id);
                debug!("raw_login2_with_unionid() -> return response: {:?}", &token);
                Ok((
                    jwt_encode_token(token)?,
                    login_user,
                    from_app.clone().unwrap(),
                ))
            }
        }
    };

    match exec_read_write_task(
        Box::new(task),
        Some(no_use),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    ) {
        Ok(res) => {
            let (token, user, app) = res;

            // 补一下老用户或者后关注某一个公众号(`to_app`)没有相关`user_info()`的问题。
            // 新用户且已经关注公众号（to_app）的，在`register()`里面就已经创建好了`user_app_info`.
            task::spawn(async move {
                let _ = post_login(
                    &user,
                    &app,
                    extract_conn(fetch_db_conn(&db_pool).unwrap()).deref_mut(),
                )
                    .await; // todo: deal with error instead of `unwrap`
            });

            Ok(token)
        }
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some(format!(
                "raw_login2_with_unionid_tx -> met err {}",
                e.to_string()
            )),
        })),
    }
}

// 登录过程没有`sudoerToken`
pub async fn post_login(
    login_user: &HtyUser,
    from_app: &HtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<()> {
    if skip_post_login() {
        debug!("post_login() ::BYPASSED::");
    } else {
        debug!("post_login() starts");
        let to_apps = AppFromTo::find_all_active_to_apps_by_from_app(&from_app.app_id, conn)?;

        for to_app in to_apps {
            let to_app_id = to_app.to_app_id;

            debug!("post_login() to_app_id -> {:?}", to_app_id);

            let existing_to_app_userinfo = UserAppInfo::verify_exist_by_app_id_and_hty_id(
                &login_user.hty_id,
                &to_app_id,
                conn,
            )?;

            debug!(
                "post_login() -> USER_EXISTING: {:?}",
                existing_to_app_userinfo
            );

            if !existing_to_app_userinfo {
                debug!("post_login() create user app info for to_app");
                let to_create_to_app_userinfo = UserAppInfo {
                    hty_id: login_user.hty_id.clone(),
                    app_id: Some(to_app_id.clone()),
                    openid: None,
                    is_registered: true,
                    id: uuid(),
                    username: None,
                    password: None,
                    meta: None,
                    created_at: Some(current_local_datetime()),
                    teacher_info: None,
                    student_info: None,
                    reject_reason: None,
                    needs_refresh: Some(false),
                    avatar_url: None,
                };

                let created_to_app_userinfo =
                    UserAppInfo::create(&to_create_to_app_userinfo, conn)?;

                debug!(
                    "post_login() create user app info for to_app successfully {:?}",
                    created_to_app_userinfo
                );

                let _ = call_refresh_openid(
                    &login_user.hty_id.clone(),
                    &created_to_app_userinfo.id.clone(),
                )
                    .await?;
            } else {
                let to_app_userinfo =
                    UserAppInfo::find_by_hty_id_and_app_id(&login_user.hty_id, &to_app_id, conn)?;

                let _ =
                    call_refresh_openid(&login_user.hty_id.clone(), &to_app_userinfo.id.clone())
                        .await?;
            }
        }
    }

    Ok(())
}

async fn call_refresh_openid(
    id_user: &String,
    id_user_app_info: &String,
) -> anyhow::Result<String> {
    let mut req_user = ReqHtyUserWithInfos {
        hty_id: Some(id_user.clone()),
        union_id: None,
        enabled: None,
        created_at: None,
        real_name: None,
        sex: None,
        mobile: None,
        infos: None,
        info_roles: None,
        settings: None,
    };

    let mut infos = Vec::new();

    let info = ReqUserAppInfo {
        id: Some(id_user_app_info.clone()),
        app_id: None,
        hty_id: None,
        openid: None,
        is_registered: false,
        username: None,
        password: None,
        roles: None,
        meta: None,
        created_at: None,
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: None,
        unread_tongzhi_count: None,
        avatar_url: None,
    };

    infos.push(info);
    req_user.infos = Some(infos);

    debug!("call_refresh_openid -> req_user: {:?}", &req_user);

    let client = reqwest::Client::new();
    let body = serde_json::to_string::<ReqHtyUserWithInfos>(&req_user).unwrap();

    debug!("call_refresh_openid -> body: {:?}", &body);

    let url = format!("{}/refresh_openid", get_uc_url());

    debug!("call_refresh_openid -> req_url: {:?}", &url);

    let resp = client
        .post(url)
        .body(body)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .send()
        .await?;

    debug!("call_refresh_openid -> resp: {:?}", &resp);

    let resp_openid: HtyResponse<String> = resp.json().await?;
    Ok(resp_openid.d.unwrap())
}

// must move in conn here because of async
pub async fn refresh_openid(
    db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(req_user): Json<ReqHtyUserWithInfos>,
) -> Json<HtyResponse<String>> {
    let id_user = req_user.hty_id.unwrap();
    let id_user_app_info = req_user
        .infos
        .clone()
        .unwrap()
        .get(0)
        .clone()
        .unwrap()
        .id
        .clone()
        .unwrap();

    match raw_refresh_openid(&id_user, &id_user_app_info, db_conn, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

async fn raw_refresh_openid(
    id_user: &String,
    id_user_app_info: &String,
    _db_conn: db::DbConn,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    let user = HtyUser::find_by_hty_id(id_user, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let user_app_info = UserAppInfo::find_by_id(
        id_user_app_info,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    let has_openid = user_app_info.openid.is_some();
    debug!(
        "refresh_openid() HAS_OPENID -> {:?} / NOT_HAS_OPENID -> {:?}",
        has_openid, !has_openid
    );

    let id_app = user_app_info.app_id.clone().unwrap();
    let union_id = user.union_id.clone().unwrap();
    let mut out_openid = "".to_string();

    let the_app = HtyApp::find_by_id(&id_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    if the_app.wx_id.is_none() || the_app.wx_id.clone().unwrap().is_empty() {
        debug!("refresh_openid() -> NO WX_ID, :::BYPASS:::");
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("refresh_openid() -> NO WX_ID, :::BYPASS:::".to_string()),
        }));
    }

    if has_openid {
        out_openid = user_app_info.openid.clone().unwrap();
    } else {
        let the_app =
            HtyApp::find_by_id(&id_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

        debug!("refresh_openid() openid -> {:?}", the_app);

        let _ = refresh_cache_and_get_wx_all_follower_openids(&the_app).await?;

        if user_app_info.openid.is_none() {
            let my_openid = find_wx_openid_by_unionid_and_hty_app(&union_id, &the_app).await?;
            out_openid = my_openid.clone();

            let to_update_user_app_info = UserAppInfo {
                hty_id: user_app_info.hty_id.clone(),
                app_id: user_app_info.app_id.clone(),
                openid: Some(my_openid),
                is_registered: true, // 激活.
                id: user_app_info.id.clone(),
                username: user_app_info.username.clone(),
                password: user_app_info.password.clone(),
                meta: user_app_info.meta.clone(),
                created_at: Some(current_local_datetime()),
                teacher_info: user_app_info.teacher_info.clone(),
                student_info: user_app_info.student_info.clone(),
                reject_reason: user_app_info.reject_reason.clone(),
                needs_refresh: user_app_info.needs_refresh.clone(),
                avatar_url: user_app_info.avatar_url.clone(),
            };

            let updated_user_app_info = UserAppInfo::update(
                &to_update_user_app_info,
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            )?;
            debug!(
                "refresh_openid() -> updated_user_app_info: {:?}",
                updated_user_app_info
            );
        }
    }

    debug!("refresh_openid() -> out_openid: {:?}", out_openid);
    Ok(out_openid)
}

pub async fn upyun_token(_sudoer: HtySudoerTokenHeader, data: String) -> Json<HtyResponse<String>> {
    let operator = get_upyun_operator().to_owned();
    let pwd = get_upyun_password().to_owned();
    let token = generate_upyun_token(&data, &operator, &pwd);

    wrap_json_ok_resp(token)
}

pub async fn upyun_token2(_sudoer: HtySudoerTokenHeader, Json(payload): Json<UpyunData>) -> Json<HtyResponse<String>> {
    let data = payload.payload.unwrap();
    let operator = get_upyun_operator().to_owned();
    let pwd = get_upyun_password().to_owned();
    let token = generate_upyun_token(&data, &operator, &pwd);

    wrap_json_ok_resp(token)
}

pub async fn wx_login(
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(in_wx_login): Json<WxLogin>,
) -> Json<HtyResponse<WxSession>> {
    debug!(":::wx_login:::");
    match raw_wx_login(&host, &in_wx_login, db_pool).await {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

async fn raw_wx_login(
    host: &HtyHostHeader,
    login: &WxLogin,
    db_pool: Arc<DbState>,
) -> anyhow::Result<WxSession> {
    let in_app = get_app_from_host(
        (*host).clone(),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    debug!(
        "{}",
        format!("raw_wx_login -> in_app -> {:?}", in_app).as_str()
    );

    // Result<WxSession, reqwest::Error>;
    let params = WxParams {
        code: Some(login.code.clone()),
        appid: Some(in_app.wx_id.unwrap().clone()),
        secret: in_app.wx_secret.clone(),
        encrypted_data: None,
        iv: None,
    };

    debug!(
        "{}",
        format!("raw_wx_login -> wx_params -> {:?}", params).as_str()
    );

    Ok(code2session(&params).await?)
}

pub async fn find_hty_resource_by_id(
    _sudoer: HtySudoerTokenHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyResource>> {
    debug!("find_hty_resource_by_id -> starts");
    match raw_find_hty_resource_by_id(&id, extract_conn(conn).deref_mut()).await {
        Ok(ok) => {
            debug!(
                "find_hty_resource_by_id -> success to find resource by id: {}, resource: {:?}!",
                id, ok
            );
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "find_hty_resource_by_id -> failed to find resource by id: {}. e: {}",
                id, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

async fn raw_find_hty_resource_by_id(
    id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyResource> {
    debug!("raw_find_hty_resource_by_id -> {:?}", id);
    Ok(HtyResource::find_by_id(id, conn)?.to_req())
}

pub async fn find_hty_resources_by_task_id(
    _sudoer: HtySudoerTokenHeader,
    Path(task_id): Path<String>,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Option<Vec<ReqHtyResource>>>> {
    debug!("find_hty_resources_by_task_id -> starts");
    match raw_find_hty_resources_by_task_id(&task_id, db_pool).await {
        Ok(ok) => {
            debug!("find_hty_resources_by_task_id -> success to find resource by id: {}, resource: {:?}!", task_id, ok);
            wrap_json_ok_resp(ok)
        }
        Err(e) => {
            error!(
                "find_hty_resource_by_id -> failed to find resource by id: {}. e: {}",
                task_id, e
            );
            wrap_json_hty_err(HtyErr {
                code: HtyErrCode::CommonError,
                reason: Some(e.to_string()),
            })
        }
    }
}

async fn raw_find_hty_resources_by_task_id(
    task_id: &String,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Option<Vec<ReqHtyResource>>> {
    let hty_resource_vs = HtyResource::find_all_by_task_id_jsonb(
        task_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;

    let ret = Some(HtyResource::convert_to_req_hty_resources(&hty_resource_vs).to_vec());

    Ok(ret)
}

pub async fn verify_jwt_token(
    host: HtyHostHeader,
    auth: AuthorizationHeader,
) -> Json<HtyResponse<()>> {
    debug!(
        "verify_jwt_token -> starts / host: {:?} / auth: {:?}",
        &host, &auth
    );
    match raw_verify_jwt_token(auth).await {
        Ok(()) => Json(HtyResponse {
            r: true,
            d: None,
            e: None,
            hty_err: None,
        }),
        Err(e) => {
            error!("verify_jwt_token -> verify failed, e: {:?}", e);
            Json(HtyResponse {
                r: false,
                d: None,
                e: Some(e.to_string()),
                hty_err: Some(HtyErr{
                    code: HtyErrCode::AuthenticationFailed,
                    reason: Some(e.to_string()),
                }),
            })
        }
    }
}

async fn raw_verify_jwt_token(auth: AuthorizationHeader) -> anyhow::Result<()> {
    let jwt_token = (*auth).clone();

    debug(format!("raw_verify_jwt_token -> token: {:?}", jwt_token).as_str());

    match verify_jwt(&jwt_token) {
        Ok(()) => {
            debug(format!("verify jwt token okay! -> {:?}", jwt_token).as_str());
            Ok(())
        }
        Err(e) => {
            debug(
                format!(
                    "raw verify jwt token error! -> TOKEN: {:?} / ERR: {:?}",
                    jwt_token, e
                )
                    .as_str(),
            );
            Err(anyhow!(e))
        }
    }
}

pub async fn login_with_cert(
    hty_host: HtyHostHeader,
    conn: db::DbConn,
    Json(req_cert): Json<ReqCert>,
) -> impl IntoResponse {
    match raw_login_with_cert(&hty_host, &req_cert, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("Login with cert success");
            (StatusCode::OK, wrap_json_ok_resp(ok))
        }
        Err(err) => {
            error!("Login with cert fail {:?}", err);
            (StatusCode::UNAUTHORIZED, wrap_json_anyhow_err(err))
        }
    }
}

fn raw_login_with_cert(
    hty_host: &HtyHostHeader,
    in_req_cert: &ReqCert,
    conn: &mut PgConnection,
) -> anyhow::Result<String> {
    debug!("raw_login_with_cert() -> htyHost: {:?}", hty_host);
    debug!("raw_login_with_cert() -> in_req_cert: {:?}", in_req_cert);
    let app_domain = (*hty_host).clone();
    let app = HtyApp::find_by_domain(&app_domain, conn)?;
    let req_cert = in_req_cert.clone();
    match verify(
        app.clone().pubkey.unwrap(),
        req_cert.encrypted_data.clone().unwrap(),
        app.clone().pubkey.unwrap(),
    ) {
        Ok(_result) => {
            let resp_token = HtyToken {
                token_id: uuid(),
                hty_id: None,
                app_id: Some(app.clone().app_id),
                ts: current_local_datetime(),
                roles: None,
                // FIXME: tags should contain `SYS_ROOT` tag
                tags: None,
            };

            save_token_with_exp_days(&resp_token, get_token_expiration_days())?;
            Ok(jwt_encode_token(resp_token)?)
        }
        Err(error) => Err(error),
    }
}

async fn generate_key_pair(
    _auth: AuthorizationHeader,
    // _sudoer: HtySudoerTokenHeader,
    _host: HtyHostHeader,
) -> Json<HtyResponse<HtyKeyPair>> {
    debug!("generate_cert_key_pair -> starts");

    match generate_cert_key_pair() {
        Ok(key_pair) => {
            debug!(
                "generate_cert_key_pair -> success generate key pair: {:?}!",
                key_pair
            );
            wrap_json_ok_resp(key_pair)
        }
        Err(e) => {
            error!(
                "generate_cert_key_pair -> failed to generate key pair, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

pub async fn get_encrypt_id_with_pubkey(
    _host: HtyHostHeader,
    req_pubkey: Json<ReqPubkey>,
    conn: db::DbConn,
) -> Json<HtyResponse<String>> {
    debug!("get_encrypt_id_with_pubkey -> starts");

    match raw_get_encrypt_id_with_pubkey(&req_pubkey, extract_conn(conn).deref_mut()) {
        Ok(ok) => {
            debug!("get_encrypt_id_with_pubkey -> success");
            wrap_json_ok_resp(ok)
        }
        Err(err) => {
            error!("get_encrypt_id_with_pubkey -> failed , e: {:?}", err);
            wrap_json_anyhow_err(err)
        }
    }
}

fn raw_get_encrypt_id_with_pubkey(
    in_req_pubkey: &Json<ReqPubkey>,
    conn: &mut PgConnection,
) -> anyhow::Result<String> {
    let req_pubkey = in_req_pubkey.clone();
    if req_pubkey.pubkey.is_none() {
        error!("raw_get_encrypt_id_with_pubkey -> pubkey null");
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("username or password error".into()),
        }));
    }

    match HtyApp::get_encrypt_app_id(&req_pubkey.pubkey.clone().unwrap(), conn) {
        Ok(encrypt_app_id) => Ok(encrypt_app_id),
        Err(_e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::CommonError,
            reason: Some("Get encrypt app id with pubkey error".into()),
        })),
    }
}

async fn wx_get_jsapi_ticket(
    _sudoer: HtySudoerTokenHeader,
    host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<String>> {
    debug!("wx_get_jsapi_ticket -> starts");
    match raw_wx_get_jsapi_ticket(host, db_pool).await {
        Ok(res) => wrap_json_ok_resp(res),
        Err(e) => {
            error!(
                "wx_get_jsapi_ticket -> failed to wx_get_jsapi_ticket, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

async fn raw_wx_get_jsapi_ticket(
    host: HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    let domain = (*host).clone();
    let app = HtyApp::find_by_domain(&domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    Ok(get_jsapi_ticket(&app).await?)
}

pub async fn wx_get_access_token(
    _sudoer: HtySudoerTokenHeader,
    hty_host: HtyHostHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<String>> {
    debug!("wx_get_access_token start");
    let ret = raw_wx_get_access_token(hty_host, db_pool).await;

    match ret {
        Ok(resp) => {
            debug!("wx_get_access_token -> {}", resp);
            wrap_json_ok_resp(resp)
        }
        Err(err) => {
            debug!("wx_get_access_token -> {}", err);
            wrap_json_anyhow_err(err)
        }
    }
}

fn get_app_from_host(host: String, conn: &mut PgConnection) -> anyhow::Result<HtyApp> {
    debug(format!("<><><>HOST -> {:?}", host).as_str());
    match HtyApp::find_by_domain(&host, conn) {
        Ok(app) => Ok(app),
        Err(err) => Err(anyhow!(err)),
    }
}

pub async fn raw_wx_get_access_token(
    host: HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<String> {
    let in_app = get_app_from_host(
        (*host).clone(),
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    )?;
    debug!("raw_wx_get_access_token start -> {:?}", in_app);
    let token = get_or_save_wx_access_token(&in_app, false).await?;
    debug!("raw_wx_get_access_token start -> {:?}", token);
    Ok(token)
}

async fn wx_identify2(
    host: HtyHostHeader,
    _db_conn: db::DbConn,
    State(db_pool): State<Arc<DbState>>,
    Json(wx_id): Json<WxId>,
) -> Json<HtyResponse<HtyToken>> {
    let domain = (*host).clone();
    match raw_wx_identify2(domain, wx_id, db_pool).await {
        Ok(res) => res,
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn raw_wx_identify2(
    domain: String,
    wx_id: WxId,
    db_pool: Arc<DbState>,
) -> anyhow::Result<Json<HtyResponse<HtyToken>>> {
    let app = HtyApp::find_by_domain(&domain, extract_conn(fetch_db_conn(&db_pool)?).deref_mut());

    if app.is_err() {
        return Ok(wrap_json_hty_err(HtyErr {
            code: HtyErrCode::DbErr,
            reason: Some(app.err().unwrap().to_string()),
        }));
    }

    match htyuc_models::wx::identify2(
        &wx_id,
        &app.unwrap().app_id,
        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
    ) {
        Ok(token) => {
            let resp = HtyResponse {
                r: true,
                d: Some(token),
                e: None,
                hty_err: None,
            };
            Ok(Json(resp))
        }
        Err(e) => Ok(wrap_json_anyhow_err(e)),
    }
}

async fn find_all_user_rels_by_params(_sudoer: HtySudoerTokenHeader,
                                      _host: HtyHostHeader,
                                      Query(params): Query<HashMap<String, String>>,
                                      conn: db::DbConn) -> Json<HtyResponse<Vec<ReqHtyUserRels>>> {
    debug!("find_all_user_rels_by_params -> starts");
    let rel_type = get_some_from_query_params::<String>("rel_type", &params);
    let from_user_id = get_some_from_query_params::<String>("from_user_id", &params);
    let to_user_id = get_some_from_query_params::<String>("to_user_id", &params);
    debug!("find_all_user_rels_by_params -> rel_type: {:?} / from_user_id: {:?} / to_user_id: {:?}", rel_type, from_user_id, to_user_id);
    match raw_find_all_user_rels_by_params(&rel_type, &from_user_id, &to_user_id, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("raw_find_all_user_rels_by_params -> failed to find rels, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_user_rels_by_params(rel_type: &Option<String>,
                                    from_user_id: &Option<String>,
                                    to_user_id: &Option<String>,
                                    conn: &mut PgConnection) -> anyhow::Result<Vec<ReqHtyUserRels>> {
    let mut res = vec![];
    let rels = HtyUserRels::find_all_with_params(rel_type, from_user_id, to_user_id, conn)?;

    debug!("raw_find_all_user_rels_by_params -> rels: {:?} / from_user_id: {:?} / to_user_id: {:?}", rels, from_user_id, to_user_id);

    for rel in rels {
        let from_real_name = HtyUser::find_by_hty_id(&rel.from_user_id, conn)?.real_name;
        let to_real_name = HtyUser::find_by_hty_id(&rel.to_user_id, conn)?.real_name;
        let item = ReqHtyUserRels {
            id: Some(rel.id.clone()),
            from_user_id: Some(rel.from_user_id.clone()),
            from_user_realname: from_real_name,
            to_user_id: Some(rel.to_user_id.clone()),
            to_user_realname: to_real_name,
            rel_type: Some(rel.rel_type.clone()),
        };
        res.push(item);
    }
    Ok(res)
}

async fn unlink_users(conn: db::DbConn, Json(req_user_rel): Json<ReqHtyUserRels>) -> Json<HtyResponse<()>> {
    debug!("unlink_users -> starts");
    match raw_unlink_users(req_user_rel, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("unlink_users -> failed to unlink user, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_unlink_users(req_user_rel: ReqHtyUserRels, conn: &mut PgConnection) -> anyhow::Result<()> {
    if req_user_rel.to_user_id.is_none() || req_user_rel.from_user_id.is_none() || req_user_rel.rel_type.is_none() {
        return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("to_user_id or from_user_id or rel_type can not be none".into())
                }));
    }
    let id_from = req_user_rel.from_user_id.unwrap();
    let id_to = req_user_rel.to_user_id.unwrap();
    let type_rel = req_user_rel.rel_type.unwrap();

    let db_item = HtyUserRels::find_by_all_col(&id_from, &id_to, &type_rel, conn)?;
    let res = HtyUserRels::delete_by_id(&db_item.id, conn);
    if res {
        Ok(())
    } else {
        return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("Failed to delete".into())
                }));
    }
}

async fn link_users(conn: db::DbConn, Json(req_user_rel): Json<ReqHtyUserRels>) -> Json<HtyResponse<()>> {
    debug!("link_users -> starts");
    match raw_link_users(req_user_rel, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("link_users -> failed to link user, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_link_users(req_user_rel: ReqHtyUserRels, conn: &mut PgConnection) -> anyhow::Result<()> {
    if req_user_rel.to_user_id.is_none() || req_user_rel.from_user_id.is_none() || req_user_rel.rel_type.is_none() {
        return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("to_user_id or from_user_id or rel_type can not be none".into())
                }));
    }
    let in_item = HtyUserRels {
        id: uuid(),
        from_user_id: req_user_rel.from_user_id.unwrap(),
        to_user_id: req_user_rel.to_user_id.unwrap(),
        rel_type: req_user_rel.rel_type.unwrap(),
    };
    let _ = HtyUserRels::create(&in_item, conn)?;
    Ok(())
}

async fn get_user_groups_by_owner_id(_root: HtySudoerTokenHeader, Path(owner_id): Path<String>, host: HtyHostHeader, conn: db::DbConn) -> Json<HtyResponse<Vec<ReqHtyUserGroup>>> {
    debug!("get_user_groups_by_owner_id -> starts");
    match raw_get_user_groups_by_owner_id(&owner_id, host, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("raw_get_user_groups_by_owner_id -> failed to find groups, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_get_user_groups_by_owner_id(owner_id: &String, host: HtyHostHeader, conn: &mut PgConnection) -> anyhow::Result<Vec<ReqHtyUserGroup>> {
    debug!("raw_get_user_groups_by_owner_id -> owner_id: {:?}", owner_id);
    let domain = (*host).clone();

    let app = HtyApp::find_by_domain(&domain, conn)?;
    debug!("raw_get_user_groups_by_owner_id -> APP {:?}", app);

    let groups = HtyUserGroup::find_by_app_id_or_owners(&owner_id, &app.app_id, conn)?;
    let res = groups.into_iter().map(|group| group.to_req()).collect();

    Ok(res)
}

async fn find_all_user_groups(
    _root: HtySudoerTokenHeader,
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<ReqHtyUserGroup>>> {
    debug!("find_all_user_groups -> starts");
    match raw_find_all_user_groups(host, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("find_all_user_groups -> failed to find groups, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_all_user_groups(
    _host: HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<ReqHtyUserGroup>> {
    let groups = HtyUserGroup::find_all_active(conn)?;
    let res = groups.into_iter().map(|group| group.to_req()).collect();

    Ok(res)
}

async fn find_group_users_by_group_id(
    _root: HtySudoerTokenHeader,
    Path(id): Path<String>,
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Vec<GroupUser>>> {
    debug!("find_group_users_by_group_id -> starts");
    match raw_find_group_users_by_group_id(&id, host, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!(
                "find_group_users_by_group_id -> failed to find group users, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_group_users_by_group_id(
    id: &String,
    _host: HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<GroupUser>> {
    let group = HtyUserGroup::find_by_id(&id, conn)?;
    let empty = vec![];
    if group.users.is_none() {
        return Ok(empty);
    }
    let users = group.users.clone().unwrap().vals;
    if users.is_none() {
        return Ok(empty);
    }
    let res = users.unwrap();
    Ok(res)
}

async fn find_user_groups_by_user_id(
    _root: HtySudoerTokenHeader,
    Path(id): Path<String>,
    host: HtyHostHeader,
    conn: db::DbConn,
) -> Json<HtyResponse<Option<Vec<ReqHtyUserGroup>>>> {
    debug!("find_user_groups_by_user_id -> starts");
    match raw_find_user_groups_by_user_id(&id, host, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!(
                "find_user_groups_by_user_id -> failed to find groups, e: {}",
                e
            );
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_user_groups_by_user_id(
    id: &String,
    host: HtyHostHeader,
    conn: &mut PgConnection,
) -> anyhow::Result<Option<Vec<ReqHtyUserGroup>>> {
    debug!("raw_find_user_groups_by_user_id -> {:?}", id);
    let domain = (*host).clone();

    let app = HtyApp::find_by_domain(&domain, conn)?;
    debug!("raw_find_user_groups_by_user_id -> APP {:?}", app);

    let some_groups = HtyUserGroup::find_by_app_id_or_users(&id, &app.app_id, conn)?;

    if some_groups.is_some() {
        let res = some_groups.unwrap().into_iter().map(|group| group.to_req()).collect();
        Ok(Some(res))
    } else {
        Ok(None)
    }
}

async fn find_user_group_by_id(
    _root: HtySudoerTokenHeader,
    Path(id): Path<String>,
    conn: db::DbConn,
) -> Json<HtyResponse<ReqHtyUserGroup>> {
    debug!("get_user_group_by_id -> starts");
    match raw_find_user_group_by_id(&id, extract_conn(conn).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("get_user_group_by_id -> failed to find group, e: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

fn raw_find_user_group_by_id(
    id: &String,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqHtyUserGroup> {
    debug!("raw_get_user_group_by_id -> {:?}", id);
    Ok(HtyUserGroup::find_by_id(id, conn)?.to_req())
}

pub fn uc_rocket(db_url: &str) -> Router {
    let db_state = DbState { pool: pool(db_url) };

    let shared_db_state = Arc::new(db_state);

    // build our application with a route
    let app = Router::new()
        .route("/api/v1/uc/index", get(index))
        .route("/api/v1/uc/register", post(register))
        .route("/api/v1/uc/notify", post(notify))
        .route("/api/v1/uc/register/rollback", post(register_rollback))
        .route("/api/v1/uc/register/verify", post(register_verify))
        .route("/api/v1/uc/create_hty_resource", post(create_hty_resource))
        .route("/api/v1/uc/update_hty_resource", post(update_hty_resource))
        .route(
            "/api/v1/uc/create_or_update_user_with_info",
            post(create_or_update_user_with_info),
        )
        .route("/api/v1/uc/create_user_group", post(create_user_group))
        .route("/api/v1/uc/find_app_by_id/{app_id}", get(find_app_by_id))
        .route("/api/v1/uc/find_app_by_domain", get(find_app_by_domain))
        .route(
            "/api/v1/uc/find_user_with_info_by_id/{id}",
            get(find_user_with_info_by_id),
        )
        .route("/api/v1/uc/find_users_in_app", post(find_users_in_app))
        .route(
            "/api/v1/uc/find_tags_by_ref_ids",
            post(find_tags_by_ref_ids),
        )
        .route("/api/v1/uc/update_user_group", post(update_user_group))
        .route(
            "/api/v1/uc/find_user_with_info_by_id_and_host/{id}",
            get(find_user_with_info_by_id_and_host),
        )
        .route(
            "/api/v1/uc/find_user_with_info_by_token",
            get(find_user_with_info_by_token),
        )
        .route("/api/v1/uc/find_hty_user_by_id/{id}", get(find_hty_user_by_id))
        .route("/api/v1/uc/find_hty_users_by_ids", post(find_hty_users_by_ids))
        .route(
            "/api/v1/uc/find_users_with_info_by_role/{role}",
            get(find_users_with_info_by_role),
        )
        .route("/api/v1/uc/find_all_user_groups", get(find_all_user_groups))
        .route(
            "/api/v1/uc/find_all_valid_teachers",
            get(find_all_valid_teachers),
        )
        .route(
            "/api/v1/uc/find_users_by_app_id/{app_id}",
            get(find_users_by_app_id),
        )
        .route("/api/v1/uc/find_users_by_domain", get(find_users_by_domain))
        .route(
            "/api/v1/uc/find_to_apps_by_domain",
            get(find_to_apps_by_domain),
        )
        .route("/api/v1/uc/sudo", post(sudo))
        .route("/api/v1/uc/sudo2/{to_user_id}", get(sudo2))
        .route("/api/v1/uc/login_with_password", post(login_with_password))
        .route("/api/v1/uc/login2_with_unionid", get(login2_with_unionid))
        .route("/api/v1/uc/wx_qr_login", post(wx_qr_login))
        .route(
            "/api/v1/uc/find_hty_resources_by_task_id/{task_id}",
            get(find_hty_resources_by_task_id),
        )
        .route(
            "/api/v1/uc/find_hty_resource_by_id/{id}",
            get(find_hty_resource_by_id),
        )
        .route(
            "/api/v1/uc/find_user_group_by_id/{id}",
            get(find_user_group_by_id),
        )
        .route(
            "/api/v1/uc/find_user_groups_by_user_id/{id}",
            get(find_user_groups_by_user_id),
        )
        .route(
            "/api/v1/uc/get_user_groups_by_owner_id/{owner_id}",
            get(get_user_groups_by_owner_id),
        )
        .route("/api/v1/uc/find_role_by_key/{key}", get(find_role_by_key))
        .route("/api/v1/uc/find_all_roles", get(find_all_roles))
        .route("/api/v1/uc/find_roles_by_app", get(find_roles_by_app))
        .route("/api/v1/uc/find_all_actions", get(find_all_actions))
        .route("/api/v1/uc/find_all_labels", get(find_all_labels))
        .route("/api/v1/uc/get_user_groups_of_current_user", get(get_user_groups_of_current_user))
        .route(
            "/api/v1/uc/update_tongzhi_status_by_id",
            post(update_tongzhi_status_by_id),
        )
        .route("/api/v1/uc/update_hty_gonggao", post(update_hty_gonggao))
        .route("/api/v1/uc/create_hty_gonggao", post(create_hty_gonggao))
        .route("/api/v1/uc/get_templates", get(get_templates))
        .route("/api/v1/uc/get_all_tags_of_the_user", get(get_all_tags_of_the_user))
        .route("/api/v1/uc/create_template", post(create_template))
        .route(
            "/api/v1/uc/create_template_data",
            post(create_template_data),
        )
        .route("/api/v1/uc/update_template", post(update_template))
        .route(
            "/api/v1/uc/update_template_data",
            post(update_template_data),
        )
        .route(
            "/api/v1/uc/delete_template/{template_id}",
            post(delete_template),
        )
        .route(
            "/api/v1/uc/delete_template_data/{template_data_id}",
            post(delete_template_data),
        )
        .route(
            "/api/v1/uc/find_all_apps_with_roles",
            get(find_all_apps_with_roles),
        )
        .route("/api/v1/uc/find_all_users", get(find_all_users))
        .route("/api/v1/uc/find_all_tags", get(find_all_tags))
        .route("/api/v1/uc/find_all_user_rels_by_params", get(find_all_user_rels_by_params))
        .route("/api/v1/uc/find_users", get(find_users))
        .route("/api/v1/uc/find_hty_users_by_keyword", get(find_hty_users_by_keyword))
        .route("/api/v1/uc/verify_user_enabled_and_registered_in_app", get(verify_user_enabled_and_registered_in_app))
        .route("/api/v1/uc/find_tongzhis", get(find_tongzhis))
        .route("/api/v1/uc/find_all_tongzhis_with_page", get(find_all_tongzhis_with_page))
        .route(
            "/api/v1/uc/delete_tongzhi_by_id/{id}",
            post(delete_tongzhi_by_id),
        )
        .route(
            "/api/v1/uc/create_or_update_apps_with_roles",
            post(create_or_update_apps_with_roles),
        )
        .route("/api/v1/uc/delete_app_by_id/{id}", post(delete_app_by_id))
        .route("/api/v1/uc/delete_user_group/{id}", post(delete_user_group))
        .route(
            "/api/v1/uc/update_official_account_openid",
            post(update_official_account_openid),
        )
        .route(
            "/api/v1/uc/find_app_with_roles/{id}",
            get(find_app_with_roles),
        )
        .route(
            "/api/v1/uc/find_group_users_by_group_id/{id}",
            get(find_group_users_by_group_id),
        )
        .route(
            "/api/v1/uc/find_tags_by_ref_id/{ref_id}",
            get(find_tags_by_ref_id),
        )
        .route(
            "/api/v1/uc/create_or_update_roles",
            post(create_or_update_roles),
        )
        .route(
            "/api/v1/uc/get_cached_kv/{key}",
            get(get_cached_kv),
        ).route("/api/v1/uc/delete_hty_resource_by_id/{id}", get(delete_hty_resource_by_id))
        .route(
            "/api/v1/uc/save_cached_kv",
            post(save_cached_kv),
        )
        .route("/api/v1/uc/find_tongzhi_by_id/{tongzhi_id}", get(find_tongzhi_by_id))
        .route("/api/v1/uc/del_cached_kv/{key}",
               get(del_cached_kv),
        )
        .route(
            "/api/v1/uc/create_or_update_actions",
            post(create_or_update_actions),
        )
        .route("/api/v1/uc/create_tag_ref", post(create_tag_ref))
        .route("/api/v1/uc/link_users", post(link_users))
        .route("/api/v1/uc/unlink_users", post(unlink_users))
        .route("/api/v1/uc/create_tongzhi", post(create_tongzhi))
        .route(
            "/api/v1/uc/create_or_update_labels",
            post(create_or_update_labels),
        )
        .route(
            "/api/v1/uc/create_or_update_userinfo_with_roles",
            post(create_or_update_userinfo_with_roles),
        )
        .route(
            "/api/v1/uc/create_or_update_tags",
            post(create_or_update_tags),
        )
        .route("/api/v1/uc/verify_jwt_token", post(verify_jwt_token))
        .route("/api/v1/uc/login_with_cert", post(login_with_cert))
        .route("/api/v1/uc/generate_key_pair", get(generate_key_pair))
        // .route("/api/v1/uc/", routes![get_encrypt_id_with_pubkey]) // !!!DANGEROUS!!!
        .route("/api/v1/uc/upyun_token", post(upyun_token))
        .route("/api/v1/uc/upyun_token2", post(upyun_token2))
        .route("/api/v1/uc/wx_identify2", post(wx_identify2))
        .route("/api/v1/uc/wx_login", post(wx_login))
        .route("/api/v1/uc/refresh_openid", post(refresh_openid))
        // .route("/api/v1/uc/wx/", routes![wx_audio_download])
        .route("/api/v1/uc/wx_get_access_token", get(wx_get_access_token))
        .route(
            "/api/v1/uc/find_hty_template_with_data_by_key_and_app_id",
            get(find_hty_template_with_data_by_key_and_app_id),
        )
        .route("/api/v1/uc/get_all_app_from_tos", get(get_all_app_from_tos))
        .route("/api/v1/uc/create_app_from_to", post(create_app_from_to))
        .route(
            "/api/v1/uc/count_unread_tongzhis_by_user_id_and_role_id",
            get(count_unread_tongzhis_by_user_id_and_role_id),
        )
        .route(
            "/api/v1/uc/delete_all_tongzhis_by_status_and_role_id_and_user_id",
            get(delete_all_tongzhis_by_status_and_role_id_and_user_id),
        )
        .route(
            "/api/v1/uc/clear_all_unread_tongzhis_by_user_id_and_role_id",
            get(clear_all_unread_tongzhis_by_user_id_and_role_id),
        )
        .route(
            "/api/v1/uc/disable_app_from_to/{disable_id}",
            post(disable_app_from_to),
        )
        .route(
            "/api/v1/uc/set_hty_resource_compression_processed/{id}",
            post(set_hty_resource_compression_processed),
        )
        .route(
            "/api/v1/uc/enable_app_from_to/{enable_id}",
            post(enable_app_from_to),
        )
        .route(
            "/api/v1/uc/delete_tag_ref/{tag_ref_id}",
            post(delete_tag_ref),
        )
        .route("/api/v1/uc/wx_get_jsapi_ticket", get(wx_get_jsapi_ticket))
        .route(
            "/api/v1/uc/update_needs_refresh_for_app",
            post(update_needs_refresh_for_app),
        )
        .route("/api/v1/uc/bulk_update_tag_ref", post(bulk_update_tag_ref))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_db_state);

    app
}
