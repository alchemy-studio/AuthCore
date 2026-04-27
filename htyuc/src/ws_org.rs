use anyhow::anyhow;
use axum::extract::{Path, Query, State};
use axum::Json;
use htycommons::common::{current_local_datetime, get_some_from_query_params, HtyErr, HtyErrCode, HtyResponse};
use htycommons::db::{extract_conn, fetch_db_conn, DbState};
use htycommons::jwt::{jwt_decode_token, jwt_encode_token};
use htycommons::redis_util::{get_token_expiration_days, save_token_with_exp_days};
use htycommons::web::{
    wrap_json_anyhow_err, wrap_json_ok_resp, AuthorizationHeader, HtySudoerTokenHeader,
    ReqOrgMember, ReqOrgRole, ReqOrganization, ReqOrgSwitch,
};
use htycommons::uuid;
use htyuc_models::models::{HtyRole, OrgMember, OrgRole, Organization, UserAppInfo};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use tracing::{debug, error};

fn current_user_app_info_id(auth: &AuthorizationHeader, conn: &mut diesel::PgConnection) -> anyhow::Result<String> {
    let token = jwt_decode_token(&(*auth).clone())?;
    let user_hty_id = token.hty_id.ok_or_else(|| anyhow!("hty_id is required in token"))?;
    let user_app_id = token.app_id.ok_or_else(|| anyhow!("app_id is required in token"))?;
    Ok(UserAppInfo::find_by_hty_id_and_app_id(&user_hty_id, &user_app_id, conn)?.id)
}

fn apply_org_context_to_token(
    token: &mut htycommons::web::HtyToken,
    target_org_id: String,
    role_keys: Vec<String>,
    new_token_id: String,
) {
    token.current_org_id = Some(target_org_id);
    token.current_org_role_keys = Some(role_keys);
    token.token_id = new_token_id;
}

pub async fn create_org(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrganization>,
) -> Json<HtyResponse<Organization>> {
    let result = (|| -> anyhow::Result<Organization> {
        let mut conn_holder = extract_conn(fetch_db_conn(&db_pool)?);
        let conn = conn_holder.deref_mut();
        let now = current_local_datetime();
        let new_org = Organization {
            id: body.id.unwrap_or_else(uuid),
            app_id: body.app_id.ok_or_else(|| anyhow!("app_id is required"))?,
            org_name: body.org_name.ok_or_else(|| anyhow!("org_name is required"))?,
            org_desc: body.org_desc,
            homepage_md: body.homepage_md,
            org_status: body.org_status.unwrap_or_else(|| "ACTIVE".to_string()),
            created_at: now,
            created_by: body.created_by,
            updated_at: None,
            updated_by: None,
            is_delete: body.is_delete.unwrap_or(false),
        };
        Organization::create(&new_org, conn)
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => {
            error!("create_org failed: {}", e);
            wrap_json_anyhow_err(e)
        }
    }
}

pub async fn update_org(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrganization>,
) -> Json<HtyResponse<Organization>> {
    let result = (|| -> anyhow::Result<Organization> {
        let mut conn_holder = extract_conn(fetch_db_conn(&db_pool)?);
        let conn = conn_holder.deref_mut();
        let org_id = body.id.ok_or_else(|| anyhow!("id is required"))?;
        let mut org = Organization::find_by_id(&org_id, conn)?;
        if let Some(v) = body.org_name {
            org.org_name = v;
        }
        if body.org_desc.is_some() {
            org.org_desc = body.org_desc;
        }
        if body.homepage_md.is_some() {
            org.homepage_md = body.homepage_md;
        }
        if let Some(v) = body.org_status {
            org.org_status = v;
        }
        if let Some(v) = body.is_delete {
            org.is_delete = v;
        }
        org.updated_at = Some(current_local_datetime());
        org.updated_by = body.updated_by;
        Organization::update(&org, conn)
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn find_org_by_id(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Path(org_id): Path<String>,
) -> Json<HtyResponse<Organization>> {
    match Organization::find_by_id(&org_id, extract_conn(fetch_db_conn(&db_pool).unwrap()).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn delete_org(
    _sudoer: HtySudoerTokenHeader,
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrganization>,
) -> Json<HtyResponse<Organization>> {
    let result = (|| -> anyhow::Result<Organization> {
        let org_id = body.id.ok_or_else(|| anyhow!("id is required"))?;
        let mut conn_holder = extract_conn(fetch_db_conn(&db_pool)?);
        let conn = conn_holder.deref_mut();
        let active_members = OrgMember::count_active_by_org_id(&org_id, conn)?;
        if active_members > 0 {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("organization has active members and cannot be deleted".to_string()),
            }));
        }
        let token = jwt_decode_token(&(*auth).clone())?;
        Organization::soft_delete_by_id(&org_id, &token.hty_id, conn)
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn list_orgs_by_app(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<HtyResponse<Vec<Organization>>> {
    let result = (|| -> anyhow::Result<Vec<Organization>> {
        let app_id = get_some_from_query_params::<String>("app_id", &params)
            .ok_or_else(|| anyhow!("app_id is required"))?;
        Organization::find_all_by_app_id(
            &app_id,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn add_org_member(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrgMember>,
) -> Json<HtyResponse<OrgMember>> {
    let result = (|| -> anyhow::Result<OrgMember> {
        let id_org = body.org_id.ok_or_else(|| anyhow!("org_id is required"))?;
        let id_role = body.role_id.ok_or_else(|| anyhow!("role_id is required"))?;
        let role = HtyRole::find_by_id(&id_role, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
        if role.is_system {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("system role cannot be assigned as org member".to_string()),
            }));
        }
        let role_in_org = OrgRole::exists_active_org_role(
            &id_org,
            &id_role,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        if !role_in_org {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("role does not belong to current organization".to_string()),
            }));
        }
        let now = current_local_datetime();
        let member = OrgMember {
            id: body.id.unwrap_or_else(uuid),
            org_id: id_org,
            user_info_id: body.user_info_id.ok_or_else(|| anyhow!("user_info_id is required"))?,
            role_id: id_role,
            member_status: body.member_status.unwrap_or_else(|| "ACTIVE".to_string()),
            joined_at: now,
            created_at: now,
            created_by: body.created_by,
            updated_at: None,
            updated_by: None,
        };
        OrgMember::create(&member, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn add_org_role(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrgRole>,
) -> Json<HtyResponse<OrgRole>> {
    let result = (|| -> anyhow::Result<OrgRole> {
        let id_org = body.org_id.ok_or_else(|| anyhow!("org_id is required"))?;
        let id_role = body.role_id.ok_or_else(|| anyhow!("role_id is required"))?;
        let role = HtyRole::find_by_id(&id_role, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
        if role.is_system {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some("system role does not need org binding".to_string()),
            }));
        }
        let now = current_local_datetime();
        let entity = OrgRole {
            id: body.id.unwrap_or_else(uuid),
            org_id: id_org,
            role_id: id_role,
            role_status: body.role_status.unwrap_or_else(|| "ACTIVE".to_string()),
            created_at: now,
            created_by: None,
            updated_at: None,
            updated_by: None,
        };
        OrgRole::create(&entity, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn remove_org_role(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrgRole>,
) -> Json<HtyResponse<usize>> {
    let result = (|| -> anyhow::Result<usize> {
        OrgRole::delete_by_org_id_and_role_id(
            &body.org_id.ok_or_else(|| anyhow!("org_id is required"))?,
            &body.role_id.ok_or_else(|| anyhow!("role_id is required"))?,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn list_org_roles(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Path(org_id): Path<String>,
) -> Json<HtyResponse<Vec<HtyRole>>> {
    let result = (|| -> anyhow::Result<Vec<HtyRole>> {
        let role_ids = OrgRole::find_active_role_ids_by_org_id(
            &org_id,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )?;
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        let all_roles = HtyRole::find_all(extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
        Ok(all_roles
            .into_iter()
            .filter(|role| role_ids.contains(&role.hty_role_id))
            .collect())
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn remove_org_member(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrgMember>,
) -> Json<HtyResponse<usize>> {
    let result = (|| -> anyhow::Result<usize> {
        OrgMember::delete_by_org_id_and_user_info_id_and_role_id(
            &body.org_id.ok_or_else(|| anyhow!("org_id is required"))?,
            &body.user_info_id.ok_or_else(|| anyhow!("user_info_id is required"))?,
            &body.role_id.ok_or_else(|| anyhow!("role_id is required"))?,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn find_org_members(
    _sudoer: HtySudoerTokenHeader,
    State(db_pool): State<Arc<DbState>>,
    Path(org_id): Path<String>,
) -> Json<HtyResponse<Vec<OrgMember>>> {
    match OrgMember::find_by_org_id(&org_id, extract_conn(fetch_db_conn(&db_pool).unwrap()).deref_mut()) {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn my_orgs(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
) -> Json<HtyResponse<Vec<Organization>>> {
    let result = (|| -> anyhow::Result<Vec<Organization>> {
        let mut conn_holder = extract_conn(fetch_db_conn(&db_pool)?);
        let conn = conn_holder.deref_mut();
        let user_info_id = current_user_app_info_id(&auth, conn)?;
        let members = OrgMember::find_by_user_info_id(&user_info_id, conn)?;
        let mut organizations_result = Vec::new();
        for member in members {
            let org = Organization::find_by_id(&member.org_id, conn)?;
            if !org.is_delete {
                organizations_result.push(org);
            }
        }
        Ok(organizations_result)
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn switch_org(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(req): Json<ReqOrgSwitch>,
) -> Json<HtyResponse<String>> {
    let result = (|| -> anyhow::Result<String> {
        let mut token = jwt_decode_token(&(*auth).clone())?;
        let target_org_id = req.org_id.ok_or_else(|| anyhow!("org_id is required"))?;
        let mut conn_holder = extract_conn(fetch_db_conn(&db_pool)?);
        let conn = conn_holder.deref_mut();
        let user_info_id = current_user_app_info_id(&auth, conn)?;
        let mut org_roles = OrgMember::find_roles_by_user_info_id_and_org_id(&user_info_id, &target_org_id, conn)?;
        let system_roles = OrgMember::find_system_roles_by_user_info_id(&user_info_id, conn)?;
        if org_roles.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::AuthenticationFailed,
                reason: Some("user has no role in target org".to_string()),
            }));
        }
        for system_role in system_roles {
            if !org_roles
                .iter()
                .any(|existing_role| existing_role.hty_role_id == system_role.hty_role_id)
            {
                org_roles.push(system_role);
            }
        }
        apply_org_context_to_token(
            &mut token,
            target_org_id,
            org_roles.into_iter().map(|role| role.role_key).collect(),
            uuid(),
        );
        save_token_with_exp_days(&token, get_token_expiration_days()?)?;
        jwt_encode_token(token).map_err(|e| anyhow!(e))
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

#[cfg(test)]
mod tests {
    use htycommons::common::current_local_datetime;
    use htycommons::web::HtyToken;

    use super::apply_org_context_to_token;

    #[test]
    fn should_apply_org_context_and_refresh_token_id() {
        let mut token = HtyToken {
            token_id: "old-token".to_string(),
            hty_id: Some("teacher-1".to_string()),
            app_id: Some("app-1".to_string()),
            ts: current_local_datetime(),
            roles: None,
            tags: None,
            current_org_id: None,
            current_org_role_keys: None,
        };

        apply_org_context_to_token(
            &mut token,
            "org-100".to_string(),
            vec!["ORG_ADMIN".to_string(), "TEACHER".to_string()],
            "new-token".to_string(),
        );

        assert_eq!(token.current_org_id, Some("org-100".to_string()));
        assert_eq!(
            token.current_org_role_keys,
            Some(vec!["ORG_ADMIN".to_string(), "TEACHER".to_string()])
        );
        assert_eq!(token.token_id, "new-token".to_string());
    }
}

pub async fn save_org_homepage(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
    Json(body): Json<ReqOrganization>,
) -> Json<HtyResponse<Organization>> {
    let result = (|| -> anyhow::Result<Organization> {
        let org_id = body.id.ok_or_else(|| anyhow!("id is required"))?;
        let homepage = body.homepage_md.ok_or_else(|| anyhow!("homepage_md is required"))?;
        let token = jwt_decode_token(&(*auth).clone())?;
        let updated_by = token.hty_id;
        Organization::update_homepage(
            &org_id,
            &homepage,
            &updated_by,
            extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
        )
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}

pub async fn get_org_homepage(
    auth: AuthorizationHeader,
    State(db_pool): State<Arc<DbState>>,
    Path(org_id): Path<String>,
) -> Json<HtyResponse<Option<String>>> {
    debug!("get_org_homepage -> token: {:?}", *auth);
    let result = (|| -> anyhow::Result<Option<String>> {
        Ok(
            Organization::find_by_id(
                &org_id,
                extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
            )?
            .homepage_md,
        )
    })();
    match result {
        Ok(ok) => wrap_json_ok_resp(ok),
        Err(e) => wrap_json_anyhow_err(e),
    }
}
