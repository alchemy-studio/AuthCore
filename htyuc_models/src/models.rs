use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::anyhow;
use chrono::NaiveDateTime;
use diesel::dsl::exists;
use diesel::expression_methods::PgTextExpressionMethods;
use diesel::pg::{Pg, PgValue};
// use diesel::serialize::IsNull;
use diesel::sql_types::Jsonb;
use diesel::{insert_into, select, sql_query, update, BelongingToDsl, BoolExpressionMethods, Connection, EqAll, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, TextExpressionMethods, OptionalExtension};
use htycommons::cert::encrypt_text_with_private_key;
use std::io::Write;
// use htycommons::pagination::LoadPaginated;
use htycommons::common::{current_local_datetime, CountResult, HtyErr, HtyErrCode};
use htycommons::db::{exec_read_write_task, CommonMeta, CommonTask, SingleVal, UNREAD, READ};
use htycommons::web::{get_uc_url, ReqHtyAction, ReqHtyLabel, ReqHtyRole, ReqHtyTag, ReqHtyTagRef};
use htycommons::{impl_jsonb_boilerplate, uuid};
use log::{debug, error};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::task;
use htycommons::models::*;
// use htycommons::logger::debug;
// use tracing_subscriber::filter::FilterExt;
// use htycommons::logger::debug;
use crate::schema::{
    actions_labels, app_from_to, apps_roles, hty_actions, hty_apps, hty_gonggao, hty_labels,
    hty_resources, hty_roles, hty_tag_refs, hty_tags, hty_template, hty_template_data, hty_tongzhi,
    hty_user_group, hty_users, roles_actions, roles_labels, user_app_info, user_info_roles, hty_user_rels,
};
// use crate::schema::hty_resources::dsl::hty_resources;
// use crate::schema::hty_labels::dsl::hty_labels;
// use crate::{role_id, tongzhi_status};
// use crate::schema::hty_apps::dsl::hty_apps;
// use crate::schema::hty_apps::pubkey;

#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_users)]
#[diesel(primary_key(hty_id))]
pub struct HtyUser {
    pub hty_id: String,
    pub union_id: Option<String>,
    pub enabled: bool,
    pub created_at: Option<NaiveDateTime>,
    pub real_name: Option<String>,
    pub sex: Option<i32>,
    pub mobile: Option<String>,
    pub settings: Option<MultiVals<UserSetting>>,
}


/*
 * - k: `receive_course_notification`
 * - v: `ON` / `OFF`
 *
 * 是否发送课程通知📢 (只限于定时通知提醒，不包括创建课程，修改课程。)
 */
#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct UserSetting {
    pub k: Option<String>,
    pub v: Option<String>,
    pub app_id: Option<String>,
    pub role_key: Option<String>,
}

impl_jsonb_boilerplate!(UserSetting);


impl HtyUser {
    pub fn find_all_hty_users_by_hty_ids(hty_ids: &Vec<String>,
                                         page: &Option<i64>,
                                         page_size: &Option<i64>,
                                         conn: &mut PgConnection) -> anyhow::Result<(Vec<HtyUser>, i64, i64)> {
        debug!("find_all_hty_users_by_hty_ids -> hty_ids: {:?}", hty_ids);

        use crate::schema::hty_users::columns::*;
        use htycommons::pagination::*;
        let mut q = hty_users::table.into_boxed();
        // todo: maybe add more conditions like order in the future.
        q = q
            .filter(hty_id.eq_any(hty_ids));

        let (resp_users, total_page, total) = q.paginate(page.clone())
            .per_page(page_size.clone())
            .load_and_count_pages(conn)?;

        debug!(
            "find_all_hty_users_by_hty_ids -> resp_users: {:?} / total_page: {:?} / total: {:?}",
            resp_users, total_page, total
        );

        Ok((resp_users, total_page, total))
    }
    pub fn find_by_openid(openid: &String, conn: &mut PgConnection) -> anyhow::Result<HtyUser> {
        debug!("find_by_openid -> openid: {:?}", openid);

        if openid.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_by_openid -> `openid` can't be NULL!".to_string())
            }));
        }

        UserAppInfo::find_hty_user_by_openid(openid, conn)
    }

    pub fn find_users_by_keyword(
        page: &Option<i64>,
        page_size: &Option<i64>,
        keyword: &Option<String>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(Vec<HtyUser>, i64, i64)> {
        use crate::schema::hty_users::columns::*;
        use htycommons::pagination::*;
        let mut q = hty_users::table.into_boxed();
        q = q.order(created_at.desc());

        if let Some(keyword_copy) = keyword.clone() {
            q = q.filter(
                real_name
                    .ilike(format!("%{}%", keyword_copy.clone()))
                    .or(mobile.like(format!("%{}%", keyword_copy.clone()))),
            );
        }

        let (resp_users, total_page, total) = q
            // .load_with_pagination(conn, page.clone(), page_size.clone())?;
            .paginate(page.clone())
            .per_page(page_size.clone())
            .load_and_count_pages(conn)?;

        debug!(
            "find_users_by_keyword -> resp_users: {:?} / total_page: {:?} / total: {:?}",
            resp_users, total_page, total
        );

        Ok((resp_users, total_page, total))

        // {
        //     Ok(res) => Ok(res),
        //     Err(e) => Err(anyhow!(HtyErr {
        //         code: HtyErrCode::DbErr,
        //         reason: Some(e.to_string()),
        //     })),
        // }
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyUser>> {
        use crate::schema::hty_users::dsl::*;
        match hty_users.order(created_at.desc()).load::<HtyUser>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(in_user: &HtyUser, conn: &mut PgConnection) -> anyhow::Result<HtyUser> {
        let result = update(hty_users::table)
            .filter(hty_users::hty_id.eq(in_user.clone().hty_id))
            .set(in_user)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_users::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_union_id(
        id_union: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        debug!("verify_exist_by_union_id -> id_union: {:?}", id_union);

        if id_union.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("verify_exist_by_union_id -> `id_union` can't be NULL!".to_string())
            }));
        }

        use crate::schema::hty_users::dsl::*;
        debug!("verify_exist_by_union_id -> {:?}", id_union);
        select(exists(hty_users.filter(union_id.eq(id_union))))
            .get_result(conn)
            .map_err(|e| {
                error!("verify_exist_by_union_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn first(conn: &mut PgConnection) -> Option<HtyUser> {
        use crate::schema::hty_users::dsl::*;
        match hty_users.first(conn) {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub fn to_req_users(in_hty_users: &Vec<HtyUser>) -> Vec<ReqHtyUser> {
        let mut out_users = vec![];
        for user in in_hty_users.iter() {
            out_users.push(ReqHtyUser {
                hty_id: Some(user.hty_id.clone()),
                union_id: user.union_id.clone(),
                enabled: Some(user.enabled),
                created_at: user.created_at.clone(),
                real_name: user.real_name.clone(),
                sex: user.sex.clone(),
                mobile: user.mobile.clone(),
                settings: user.settings.clone(),
            })
        }
        out_users
    }

    // todo: change return to <Vec<HtyUser, Vec<UserAppInfo>>
    // ref: https://docs.rs/diesel/0.8.0/diesel/associations/index.html
    pub fn find_all_by_app_id(
        in_app_id: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyUser>> {
        debug!("find_all_by_app_id -> in_app_id: {:?}", in_app_id);

        if in_app_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_all_by_app_id -> `in_app_id` can't be NULL!".to_string())
            }));
        }

        use crate::schema::user_app_info::dsl::*;
        match user_app_info
            .filter(app_id.eq(in_app_id))
            .inner_join(hty_users::table)
            .select(hty_users::all_columns)
            .load::<HtyUser>(conn)
        {
            Ok(users) => Ok(users),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn first_with_info(conn: &mut PgConnection) -> Option<Vec<(UserAppInfo, HtyUser)>> {
        match HtyUser::first(conn) {
            Some(u) => {
                match UserAppInfo::belonging_to(&u)
                    .inner_join(hty_users::table)
                    .select((user_app_info::all_columns, hty_users::all_columns))
                    .load::<(UserAppInfo, HtyUser)>(conn)
                {
                    Ok(t) => Some(t),
                    Err(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn create(user: &HtyUser, conn: &mut PgConnection) -> anyhow::Result<HtyUser> {
        use crate::schema::hty_users::dsl::*;
        match insert_into(hty_users).values(user).get_result(conn) {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_union_id(id_union: &String, conn: &mut PgConnection) -> anyhow::Result<HtyUser> {
        debug!("find_by_union_id -> id_union: {:?}", id_union);

        if id_union.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_by_union_id -> `id_union` can't be NULL!".to_string())
            }));
        }

        match hty_users::table
            .filter(hty_users::union_id.eq(id_union))
            .first::<HtyUser>(conn)
        {
            Ok(user) => Ok(user),
            Err(_e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_by_union_id Not Found!".to_string()),
            })),
        }
    }

    pub fn find_opt_by_union_id(
        id_union: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Option<HtyUser>> {
        debug!("find_opt_by_union_id -> id_union: {:?}", id_union);

        if id_union.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_opt_by_union_id -> `id_union` can't be NULL!".to_string()),
            }));
        }

        use crate::schema::hty_users::dsl::*;
        Ok(hty_users
            .filter(union_id.eq(id_union))
            .first::<HtyUser>(conn)
            .optional()?)
    }

    pub fn find_or_create_disabled_by_union_id(
        id_union: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(HtyUser, bool)> {
        debug!(
            "find_or_create_disabled_by_union_id -> id_union: {:?}",
            id_union
        );

        if id_union.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(
                    "find_or_create_disabled_by_union_id -> `id_union` can't be NULL!".to_string()
                ),
            }));
        }

        conn.transaction::<(HtyUser, bool), anyhow::Error, _>(|conn| {
            // Avoid duplicate user creation under concurrency.
            sql_query("SELECT pg_advisory_xact_lock(hashtext($1))")
                .bind::<diesel::sql_types::Text, _>(id_union)
                .execute(conn)?;

            if let Some(existing) = HtyUser::find_opt_by_union_id(id_union, conn)? {
                return Ok((existing, false));
            }

            let to_create_user = HtyUser {
                hty_id: uuid(),
                union_id: Some(id_union.to_string()),
                enabled: false,
                created_at: Some(current_local_datetime()),
                real_name: None,
                sex: None,
                mobile: None,
                settings: None,
            };

            match HtyUser::create(&to_create_user, conn) {
                Ok(created) => Ok((created, true)),
                Err(e) => {
                    if is_unique_violation_err(&e) {
                        if let Some(existing) = HtyUser::find_opt_by_union_id(id_union, conn)? {
                            return Ok((existing, false));
                        }
                    }
                    Err(e)
                }
            }
        })
    }

    pub fn find_by_mobile(
        in_mobile: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Option<HtyUser>> {
        debug!("find_by_mobile -> mobile: {:?}", in_mobile);

        if in_mobile.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_by_mobile -> `mobile` can't be NULL!".to_string()),
            }));
        }

        use crate::schema::hty_users::dsl::*;
        Ok(hty_users
            .filter(mobile.eq(in_mobile))
            .first::<HtyUser>(conn)
            .optional()?)
    }

    pub fn find_by_hty_id(id_hty: &str, conn: &mut PgConnection) -> anyhow::Result<HtyUser> {
        debug!("find_by_hty_id -> id_hty: {:?}", id_hty);

        if id_hty.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_by_id -> `uid` can't be NULL!".to_string())
            }));
        }

        use crate::schema::hty_users::dsl::*;
        match hty_users.find(id_hty).get_result::<HtyUser>(conn) {
            Ok(user) => Ok(user),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_hty_id(in_hty_id: &str, conn: &mut PgConnection) -> anyhow::Result<i32> {
        debug!("delete_by_hty_id -> in_hty_id: {:?}", in_hty_id);

        if in_hty_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("delete_by_id -> `in_hty_id` can't be NULL!".to_string())
            }));
        }

        use crate::schema::hty_users::dsl::*;
        match diesel::delete(hty_users.find(in_hty_id)).execute(conn) {
            Ok(num) => Ok(num as i32),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn create_with_info_with_tx(
        user: &HtyUser,
        info: &Option<UserAppInfo>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<String> {
        let mut params = HashMap::new();

        params.insert("params".to_string(), (user.clone(), info.clone()));

        let task = move |in_params: Option<HashMap<String, (HtyUser, Option<UserAppInfo>)>>,
                         conn: &mut PgConnection|
                         -> anyhow::Result<String> {
            let params_map = in_params
                .ok_or_else(|| anyhow::anyhow!("in_params is required"))?;
            let (in_user, in_info) = params_map.get("params")
                .ok_or_else(|| anyhow::anyhow!("params key is required"))?
                .clone();
            match HtyUser::create_with_info(&in_user, &in_info, conn) {
                Ok(hty_id) => Ok(hty_id),
                Err(e) => Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string())
                })),
            }
        };

        exec_read_write_task(Box::new(task), Some(params), conn)
    }

    pub fn create_with_info(
        user: &HtyUser,
        info: &Option<UserAppInfo>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<String> {
        if user.hty_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("hty_id is empty!".to_string()),
            }));
        }

        let hty_id = user.hty_id.clone();

        let mut c_user = user.clone();
        c_user.hty_id = hty_id.clone();

        let u = HtyUser::create(&c_user, conn)?;

        match info {
            Some(in_info) => {
                let mut c_info = in_info.clone();

                c_info.hty_id = hty_id.clone();
                c_info.id = uuid();

                match UserAppInfo::create(&c_info, conn) {
                    Ok(_) => Ok(u.hty_id.clone()),
                    Err(e) => Err(anyhow!(e)),
                }
            }
            None => Ok(u.hty_id.clone()),
        }
    }

    pub fn update_user_with_info_with_tx(
        user: &Option<HtyUser>,
        info: &Option<UserAppInfo>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<String> {
        let mut params = HashMap::new();

        params.insert("params".to_string(), (user.clone(), info.clone()));

        let task =
            move |in_params: Option<HashMap<String, (Option<HtyUser>, Option<UserAppInfo>)>>,
                  conn: &mut PgConnection|
                  -> anyhow::Result<String> {
                let (in_user, in_info) = in_params
                    .ok_or(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some(
                            "update_user_with_info_with_tx(): in_params is empty!!!".to_string(),
                        ),
                    }))?
                    .get("params")
                    .ok_or(anyhow!(HtyErr {
                        code: HtyErrCode::NullErr,
                        reason: Some(
                            "update_user_with_info_with_tx(): in_params is empty!!!".to_string(),
                        ),
                    }))?
                    .clone();
                HtyUser::update_user_with_info(&in_user, &in_info, conn)
            };

        exec_read_write_task(Box::new(task), Some(params), conn)
    }

    pub fn update_user_with_info(
        user: &Option<HtyUser>,
        info: &Option<UserAppInfo>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<String> {
        // todo: add test
        if user.is_none() && info.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("no user or info provided".to_string()),
            }));
        }

        let in_user: HtyUser;

        if let Some(user_val) = user.clone() {
            in_user = user_val;
        } else {
            let info_val = info.clone()
                .ok_or_else(|| anyhow::anyhow!("info is required when user is None"))?;
            match HtyUser::find_by_hty_id(info_val.hty_id.as_str(), conn) {
                Ok(u) => {
                    in_user = u;
                }
                Err(e) => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::DbErr,
                        reason: Some(e.to_string()),
                    }));
                }
            };
        }

        let query = diesel::update(&in_user).set(&in_user);

        match query.execute(conn) {
            Ok(_) => match info {
                Some(in_info) => match in_info.find_self(conn) {
                    Ok(self_info) => match diesel::update(&self_info).set(in_info).execute(conn) {
                        Ok(_) => Ok(in_user.hty_id.clone()),
                        Err(e) => Err(anyhow!(HtyErr {
                            code: HtyErrCode::DbErr,
                            reason: Some(e.to_string()),
                        })),
                    },
                    Err(e) => Err(anyhow!(e)),
                },
                None => Ok(in_user.hty_id.clone()),
            },
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_user_with_all_info(hty_id: &str, conn: &mut PgConnection) -> anyhow::Result<i32> {
        match UserAppInfo::delete_all_by_hty_id(hty_id, conn) {
            Ok(r) => match HtyUser::delete_by_hty_id(hty_id, conn) {
                Ok(_) => Ok(r as i32),
                Err(e) => Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })),
            },
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn all_users_by_app_domain(
        domain: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyUser>> {
        match hty_apps::table
            .filter(hty_apps::domain.eq(domain))
            .first::<HtyApp>(conn)
        {
            Ok(hty_app) => {
                match UserAppInfo::belonging_to(&hty_app)
                    .inner_join(hty_users::table)
                    .select(hty_users::all_columns)
                    .load::<HtyUser>(conn)
                {
                    Ok(users) => Ok(users),
                    Err(e) => Err(anyhow!(HtyErr {
                        code: HtyErrCode::DbErr,
                        reason: Some(e.to_string()),
                    })),
                }
            }
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn to_req_user(&self) -> ReqHtyUser {
        ReqHtyUser {
            hty_id: Some(self.hty_id.clone()),
            union_id: self.union_id.clone(),
            enabled: Some(self.enabled),
            created_at: self.created_at.clone(),
            real_name: self.real_name.clone(),
            sex: self.sex.clone(),
            mobile: self.mobile.clone(),
            settings: self.settings.clone(),
        }
    }

    pub fn info(&self, app_id: &String, conn: &mut PgConnection) -> anyhow::Result<UserAppInfo> {
        UserAppInfo::find_by_hty_id_and_app_id(&self.hty_id, app_id, conn)
    }
}

fn is_unique_violation_err(err: &anyhow::Error) -> bool {
    if let Some(hty_err) = err.downcast_ref::<HtyErr>() {
        if hty_err.code == HtyErrCode::DbErr {
            if let Some(reason) = &hty_err.reason {
                let reason_lower = reason.to_lowercase();
                return reason_lower.contains("duplicate key")
                    || reason_lower.contains("unique constraint")
                    || reason_lower.contains("unique violation");
            }
        }
    }
    false
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqLogin {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqQRCode {
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpyunData {
    pub payload: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqPubkey {
    pub pubkey: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqCert {
    pub encrypted_data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqHtyApp {
    pub app_id: Option<String>,
    pub wx_id: Option<String>,
    pub wx_secret: Option<String>,
    pub domain: Option<String>,
    pub app_desc: Option<String>,
    pub app_status: Option<String>,
    pub role_ids: Option<Vec<String>>,
    pub roles: Option<Vec<HtyRole>>,
    pub gonggaos: Option<Vec<HtyGongGao>>,
    pub tags: Option<Vec<HtyTag>>,
    pub pubkey: Option<String>,
    pub privkey: Option<String>,
    pub needs_refresh: Option<bool>,
    pub is_wx_app: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromSqlRow, PartialEq)]
pub struct ReqHtyUserGroup {
    pub id: Option<String>,
    pub users: Option<MultiVals<GroupUser>>,
    pub group_type: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub app_id: Option<String>,
    pub group_name: Option<String>,
    pub is_delete: Option<bool>,
    pub group_desc: Option<String>,
    pub parent_id: Option<String>,
    pub owners: Option<MultiVals<GroupUser>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct ReqHtyTemplate<T: Clone + Debug + Serialize + DeserializeOwned> {
    pub id: Option<String>,
    pub template_key: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub template_desc: Option<String>,
    pub datas: Option<Vec<ReqHtyTemplateData<T>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct ReqHtyTemplateData<T: Clone + Debug + Serialize + DeserializeOwned> {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub template_id: Option<String>,
    pub template_val: Option<String>,
    pub template_text: Option<SingleVal<T>>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
}

impl<T: Clone + Debug + Serialize + DeserializeOwned> ReqHtyTemplateData<T> {
    pub fn to_db_struct(&self) -> anyhow::Result<HtyTemplateData<T>> {
        if self.id.is_none()
            || self.app_id.is_none()
            || self.template_id.is_none()
            || self.created_by.is_none()
            || self.created_at.is_none()
        {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some(
                    "id or app_id or template_id or created_by or created_at is none".into()
                )
            }));
        }

        let to_db: HtyTemplateData<T> = HtyTemplateData {
            id: self.id.clone().expect("id should not be None after check"),
            app_id: self.app_id.clone().expect("app_id should not be None after check"),
            template_id: self.template_id.clone().expect("template_id should not be None after check"),
            template_val: self.template_val.clone(),
            template_text: self.template_text.clone(),
            created_at: self.created_at.clone().expect("created_at should not be None after check"),
            created_by: self.created_by.clone().expect("created_by should not be None after check"),
        };

        Ok(to_db)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqHtyUser {
    pub hty_id: Option<String>,
    pub union_id: Option<String>,
    pub enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub real_name: Option<String>,
    pub sex: Option<i32>,
    pub mobile: Option<String>,
    pub settings: Option<MultiVals<UserSetting>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqHtyUserWithInfos {
    pub hty_id: Option<String>,
    pub union_id: Option<String>,
    pub enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub real_name: Option<String>,
    pub sex: Option<i32>,
    pub mobile: Option<String>,
    pub infos: Option<Vec<ReqUserAppInfo>>,
    pub info_roles: Option<Vec<ReqUserInfoRole>>,
    pub settings: Option<MultiVals<UserSetting>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqAppFromTo {
    pub id: Option<String>,
    pub from_app_id: Option<String>,
    pub to_app_id: Option<String>,
    pub is_enabled: Option<bool>,
}

impl ReqHtyUser {
    pub fn to_hty_user(&self) -> Result<HtyUser, HtyErr> {
        let hty_id;

        match self.hty_id.clone() {
            Some(v) => {
                hty_id = v;
            }
            None => {
                return Err(HtyErr {
                    code: HtyErrCode::NullErr,
                    reason: Some("hty_id is null".into()),
                });
            }
        }

        let enabled;

        match self.enabled.clone() {
            Some(v) => {
                enabled = v;
            }
            None => enabled = false,
        }
        Ok(HtyUser {
            hty_id,
            union_id: self.union_id.clone(),
            enabled,
            created_at: self.created_at.clone(),
            real_name: self.real_name.clone(),
            sex: self.sex.clone(),
            mobile: self.mobile.clone(),
            settings: self.settings.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqRegister {
    pub unionid: Option<String>,
    pub openid: Option<String>,
    pub real_name: Option<String>,
    pub sex: Option<i32>,
    pub mobile: Option<String>,
    pub role: Option<String>,
    pub teacher_id: Option<String>,
    pub meta: Option<MetaUserAppInfo>,
    pub teacher_info: Option<TeacherInfo>,
    pub student_info: Option<StudentInfo>,
    pub enabled: Option<bool>,
    pub user_settings: Option<MultiVals<UserSetting>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqUserIdWithAppId {
    pub app_id: Option<String>,
    pub hty_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqUpdateTongzhiStatus {
    pub tongzhi_id: Option<String>,
    pub tongzhi_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqHtyGongGao {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub gonggao_status: Option<String>,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqRegisterVerify {
    pub hty_id: Option<String>,
    pub validate: Option<bool>,
    pub reject_reason: Option<String>,
    pub app_id: Option<String>,
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_gonggao)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
pub struct HtyGongGao {
    pub id: String,
    pub app_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub gonggao_status: Option<String>,
    pub content: Option<String>,
}

impl HtyGongGao {
    pub fn create(in_item: &HtyGongGao, conn: &mut PgConnection) -> anyhow::Result<HtyGongGao> {
        use crate::schema::hty_gonggao::dsl::*;
        match insert_into(hty_gonggao).values(in_item).get_result(conn) {
            Ok(v) => Ok(v),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_by_id(id_gonggao: &String, conn: &mut PgConnection) -> anyhow::Result<HtyGongGao> {
        match hty_gonggao::table
            .filter(hty_gonggao::id.eq(id_gonggao))
            .select(hty_gonggao::all_columns)
            .first::<HtyGongGao>(conn)
        {
            Ok(gonggao) => Ok(gonggao),
            Err(e) => Err({
                error!("find_by_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_by_app_id(
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyGongGao>> {
        match hty_gonggao::table
            .filter(hty_gonggao::app_id.eq(id_app))
            .load::<HtyGongGao>(conn)
        {
            Ok(gonggao) => Ok(gonggao),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn update(in_item: &HtyGongGao, conn: &mut PgConnection) -> anyhow::Result<HtyGongGao> {
        let result = update(hty_gonggao::table)
            .filter(hty_gonggao::id.eq(in_item.clone().id))
            .set(in_item)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_gonggao::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = user_app_info)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(HtyUser, foreign_key = hty_id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
pub struct UserAppInfo {
    pub hty_id: String,
    pub app_id: Option<String>,
    pub openid: Option<String>,
    pub is_registered: bool,
    pub id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub meta: Option<MetaUserAppInfo>,
    pub created_at: Option<NaiveDateTime>,
    pub teacher_info: Option<TeacherInfo>,
    pub student_info: Option<StudentInfo>,
    pub reject_reason: Option<String>,
    pub needs_refresh: Option<bool>,
    pub avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RespUserStatusByPhone {
    pub enabled: bool,
    pub is_registered: bool,
}

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct TeacherInfo {
    pub academic: Option<String>,
    pub experience: Option<String>,
}

impl_jsonb_boilerplate!(TeacherInfo);

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct StudentInfo {
    pub experience: Option<String>,
}

impl_jsonb_boilerplate!(StudentInfo);

impl UserAppInfo {
    pub fn find_by_id(
        id_user_app_info: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        use crate::schema::user_app_info::dsl::*;
        match user_app_info
            .find(id_user_app_info)
            .get_result::<UserAppInfo>(conn)
        {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(
        in_user_app_info: &UserAppInfo,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        debug!("UPDATE USER -> {:?}", &in_user_app_info);

        let result = update(user_app_info::table)
            .filter(user_app_info::id.eq(in_user_app_info.clone().id))
            .set(in_user_app_info)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(user_app_info::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn convert_to_req_user_info_vs(infos: &Vec<UserAppInfo>) -> Vec<ReqUserAppInfo> {
        let mut out_vs = vec![];
        for info in infos.iter() {
            out_vs.push(info.to_req().clone());
        }
        out_vs
    }

    pub fn to_req(&self) -> ReqUserAppInfo {
        ReqUserAppInfo {
            id: Some(self.id.clone()),
            app_id: self.app_id.clone(),
            hty_id: Some(self.hty_id.clone()),
            openid: self.openid.clone(),
            is_registered: self.is_registered,
            username: self.username.clone(),
            password: self.password.clone(),
            roles: None,
            meta: self.meta.clone(),
            created_at: self.created_at.clone(),
            teacher_info: self.teacher_info.clone(),
            student_info: self.student_info.clone(),
            reject_reason: self.reject_reason.clone(),
            needs_refresh: self.needs_refresh.clone(),
            unread_tongzhi_count: None,
            avatar_url: self.avatar_url.clone(),
        }
    }

    pub fn find_linked_roles(&self, conn: &mut PgConnection) -> anyhow::Result<Vec<HtyRole>> {
        let res = UserInfoRole::belonging_to(self)
            .inner_join(hty_roles::table)
            .select(hty_roles::all_columns)
            .load::<HtyRole>(conn)?;
        Ok(res)
    }

    pub fn find_by_username_and_app_id(
        username: &String,
        app_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        Ok(user_app_info::table
            .filter(
                user_app_info::app_id
                    .eq(app_id)
                    .and(user_app_info::username.eq(username)),
            )
            .first::<UserAppInfo>(conn)?)
    }

    //TODO: Add find_by_app_id function

    pub fn verify_unique_username_by_app_id(
        user_info: &ReqUserAppInfo,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::user_app_info::dsl::*;
        select(exists(
            user_app_info.filter(
                app_id
                    .eq(id_app)
                    .and(username.eq(user_info.username.clone())),
            ),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }

    pub fn verify_exist_by_app_id_and_hty_id(
        id_hty: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        debug!(
            "verify_exist_by_app_id_and_hty_id -> id_hty: {:?} / id_app: {:?}",
            id_hty, id_app
        );

        if id_hty.is_empty() || id_app.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(
                    "verify_exist_by_app_id_and_hty_id -> `id_hty` or `id_app` can't be NULL!"
                        .to_string()
                )
            }));
        }

        use crate::schema::user_app_info::dsl::*;

        let result = select(exists(
            user_app_info.filter(app_id.eq(id_app).and(hty_id.eq(id_hty))),
        ))
            .get_result::<bool>(conn);

        if result.is_err() {
            Ok(false)
        } else {
            Ok(result.unwrap_or(false))
        }
    }

    pub fn find_by_req_info(
        req_info: &ReqUserAppInfo,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        if req_info.id.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("user app info id is null!".to_string()),
            }));
        }

        match user_app_info::table
            .filter(user_app_info::id.eq(req_info.id.clone().expect("id should not be None after check")))
            .first::<UserAppInfo>(conn)
        {
            Ok(info) => Ok(info),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_self(&self, conn: &mut PgConnection) -> anyhow::Result<UserAppInfo> {
        match user_app_info::table
            .filter(user_app_info::id.eq(self.id.clone()))
            .first::<UserAppInfo>(conn)
        {
            Ok(info) => Ok(info),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_user_infos_by_hty_id(
        id_hty: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<UserAppInfo>> {
        debug!("find_all_user_infos_by_hty_id -> id_hty: {:?}", id_hty);

        if id_hty.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_all_user_infos_by_hty_id -> `id_hty` can't be NULL!".to_string())
            }));
        }

        match user_app_info::table
            .filter(user_app_info::hty_id.eq(id_hty.to_string()))
            .load::<UserAppInfo>(conn)
        {
            Ok(infos) => Ok(infos),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_by_app_id(
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<UserAppInfo>> {
        debug!("find_all_by_app_id -> id_app: {:?}", id_app);

        if id_app.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("find_all_by_app_id -> `id_app` can't be NULL!".to_string())
            }));
        }

        match user_app_info::table
            .filter(user_app_info::app_id.eq(id_app.to_string()))
            .load::<UserAppInfo>(conn)
        {
            Ok(infos) => Ok(infos),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_hty_id_and_app_id(
        hty_id: &String,
        app_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        debug!(
            "find_by_hty_id_and_app_id -> hty_id / {:?} / app_id / {:?}",
            hty_id, app_id
        );

        if hty_id.is_empty() || app_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(
                    "find_by_hty_id_and_app_id -> `hty_id` or `app_id` can't be NULL!".to_string()
                )
            }));
        }

        match user_app_info::table
            .filter(
                user_app_info::hty_id
                    .eq(hty_id)
                    .and(user_app_info::app_id.eq(app_id)),
            )
            .first::<UserAppInfo>(conn)
        {
            Ok(info) => {
                debug!("find_by_hty_id_and_app_id / find app info -> {:?}", info);
                Ok(info)
            }
            Err(e) => {
                error!("find_by_hty_id_and_app_id / *WARNING* (THIS IS OK BECAUSE THIS USER MAY NOT HAVE THIS TO_APP) -> {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                }))
            }
        }
    }

    pub fn find_opt_by_hty_id_and_app_id(
        in_hty_id: &str,
        in_app_id: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Option<UserAppInfo>> {
        debug!(
            "find_opt_by_hty_id_and_app_id -> hty_id / {:?} / app_id / {:?}",
            in_hty_id, in_app_id
        );

        if in_hty_id.is_empty() || in_app_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(
                    "find_opt_by_hty_id_and_app_id -> `hty_id` or `app_id` can't be NULL!"
                        .to_string(),
                ),
            }));
        }

        use crate::schema::user_app_info::dsl::*;

        Ok(user_app_info
            .filter(hty_id.eq(in_hty_id).and(app_id.eq(in_app_id)))
            .first::<UserAppInfo>(conn)
            .optional()?)
    }

    pub fn find_hty_user_by_openid(
        openid: &str,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyUser> {
        debug!("find_hty_user_by_openid -> openid: {:?}", openid);

        if openid.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("OPEN_ID can't be NULL!".to_string())
            }));
        }

        match user_app_info::table
            .filter(user_app_info::openid.eq(openid))
            .select(user_app_info::hty_id)
            .first::<String>(conn)
        {
            Ok(hty_id) => {
                debug!("find_hty_user_by_openid-> user: {:?}", openid);
                HtyUser::find_by_hty_id(hty_id.as_str(), conn)
            }
            Err(e) => {
                debug!("find_hty_user_by_openid-> err: {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                }))
            }
        }
    }

    pub fn create(info: &UserAppInfo, conn: &mut PgConnection) -> anyhow::Result<UserAppInfo> {
        // debug!("create -> info: {:?}", info);
        //
        // if info.is_empty() {
        //     return Err(anyhow!(HtyErr {code: HtyErrCode::DbErr, reason: Some("info can't be NULL!".to_string())}));
        // }

        use crate::schema::user_app_info::dsl::*;
        match insert_into(user_app_info).values(info).get_result(conn) {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete(in_id: &str, conn: &mut PgConnection) -> anyhow::Result<i32> {
        debug!("delete -> in_id: {:?}", in_id);

        if in_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("in_id can't be NULL!".to_string())
            }));
        }

        use crate::schema::user_app_info::dsl::*;
        match diesel::delete(user_app_info.find(in_id)).execute(conn) {
            Ok(ok) => Ok(ok as i32),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string())
            })),
        }
    }

    pub fn delete_by_hty_id_and_app_id(
        in_hty_id: &String,
        in_app_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<i32> {
        debug!(
            "delete_by_hty_id_and_app_id -> in_hty_id: {:?} / in_app_id: {:?}",
            in_hty_id, in_app_id
        );

        if in_hty_id.is_empty() || in_app_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("`in_hty_id` or `in_app_id` can't be NULL!".to_string())
            }));
        }

        use crate::schema::user_app_info::dsl::*;
        match diesel::delete(user_app_info.filter(hty_id.eq(in_hty_id).and(app_id.eq(in_app_id))))
            .execute(conn)
        {
            Ok(num) => Ok(num as i32),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all_by_hty_id(in_hty_id: &str, conn: &mut PgConnection) -> anyhow::Result<i32> {
        debug!("delete_by_hty_id_and_app_id -> in_hty_id: {:?}", in_hty_id);

        if in_hty_id.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some("`in_hty_id` can't be NULL!".to_string())
            }));
        }

        use crate::schema::user_app_info::dsl::*;
        match diesel::delete(user_app_info.filter(hty_id.eq(in_hty_id))).execute(conn) {
            Ok(num) => Ok(num as i32),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn roles_by_id(&self, conn: &mut PgConnection) -> anyhow::Result<Option<Vec<HtyRole>>> {
        let info_role_list = UserInfoRole::belonging_to(self).load::<UserInfoRole>(conn)?;
        let res = info_role_list
            .iter()
            .map(|info_role| {
                let role = HtyRole::find_by_id(info_role.role_id.as_str(), conn).ok();
                role
            })
            .collect();
        return Ok(res);
    }

    pub fn req_roles_by_id(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Option<Vec<ReqHtyRole>>> {
        let role_list = self.find_linked_roles(conn)?;

        let res: Option<Vec<ReqHtyRole>> = role_list
            .iter()
            .map(|role| {
                let (actions, labels) = role.find_linked_action_and_label(conn).ok()?;
                let req_action_list: Option<Vec<ReqHtyAction>> = actions
                    .iter()
                    .map(|action| {
                        let req_action = action.to_req_action(conn).ok();
                        req_action
                    })
                    .collect();
                let req_label_list: Vec<ReqHtyLabel> =
                    labels.iter().map(|label| label.to_req_label()).collect();
                Some(ReqHtyRole {
                    hty_role_id: Some(role.clone().hty_role_id),
                    user_app_info_id: Some(self.clone().id),
                    app_ids: if let Some(app_id) = self.app_id.clone() {
                        Some(vec![app_id])
                    } else {
                        None
                    },
                    role_key: Some(role.clone().role_key),
                    role_desc: role.clone().role_desc,
                    role_status: Some(role.role_status.clone()),
                    labels: Some(req_label_list),
                    actions: Some(req_action_list?),
                    style: role.style.clone(),
                    role_name: role.role_name.clone(),
                })
            })
            .collect();
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqHtyTongzhi {
    pub tongzhi_id: Option<String>,
    pub app_id: Option<String>,
    pub tongzhi_type: Option<String>,
    pub tongzhi_status: Option<String>,
    pub send_from: Option<String>,
    pub send_to: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub content: Option<CommonTongzhiContent>,
    pub meta: Option<TongzhiMeta>,
    pub role_id: Option<String>,
    pub push_info: Option<PushInfo>,
}

impl ReqHtyTongzhi {
    pub fn to_db_struct(&self) -> HtyTongzhi {
        HtyTongzhi {
            tongzhi_id: self.tongzhi_id.clone().expect("tongzhi_id is required"),
            app_id: self.app_id.clone().expect("app_id is required"),
            tongzhi_type: self.tongzhi_type.clone().expect("tongzhi_type is required"),
            tongzhi_status: self.tongzhi_status.clone().expect("tongzhi_status is required"),
            send_from: self.send_from.clone(),
            send_to: self.send_to.clone().expect("send_to is required"),
            created_at: self.created_at.clone().expect("created_at is required"),
            content: self.content.clone(),
            meta: self.meta.clone(),
            role_id: self.role_id.clone(), // 一个用户可能有多种角色,每种角色接受各自的通知.(例如:作为学生接收学生通知;作为老师接收老师通知.)
            push_info: self.push_info.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqUserAppInfo {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub hty_id: Option<String>,
    pub openid: Option<String>,
    pub is_registered: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub roles: Option<Vec<ReqHtyRole>>,
    pub meta: Option<MetaUserAppInfo>,
    pub created_at: Option<NaiveDateTime>,
    pub teacher_info: Option<TeacherInfo>,
    pub student_info: Option<StudentInfo>,
    pub reject_reason: Option<String>,
    pub needs_refresh: Option<bool>,
    pub unread_tongzhi_count: Option<i32>,
    pub avatar_url: Option<String>,
}

impl ReqUserAppInfo {
    pub fn to_user_app_info_and_pick_id(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserAppInfo> {
        let db_info = UserAppInfo::find_by_req_info(&self, conn);
        match db_info {
            Ok(info) => {
                let mut mut_info = self.clone();
                mut_info.id = Some(info.id.clone());
                mut_info.to_user_app_info()
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn to_user_app_info(&self) -> anyhow::Result<UserAppInfo> {
        if self.hty_id.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("hty_id is null!".into()),
            }));
        }

        if self.id.is_none() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("info id is null!".into()),
            }));
        }

        Ok(UserAppInfo {
            hty_id: self.hty_id.clone().expect("hty_id should not be None after check"),
            app_id: self.app_id.clone(),
            openid: self.openid.clone(),
            is_registered: self.is_registered,
            id: self.id.clone().expect("id should not be None after check"),
            username: self.username.clone(),
            password: self.password.clone(),
            meta: self.meta.clone(),
            created_at: self.created_at.clone(),
            teacher_info: self.teacher_info.clone(),
            student_info: self.student_info.clone(),
            reject_reason: self.reject_reason.clone(),
            needs_refresh: self.needs_refresh.clone(),
            avatar_url: self.avatar_url.clone(),
        })
    }
}

#[derive(
Identifiable,
AsChangeset,
PartialEq,
// Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
)]
#[diesel(table_name = hty_apps)]
#[diesel(primary_key(app_id))]
pub struct HtyApp {
    pub app_id: String,
    pub wx_secret: Option<String>,
    pub domain: Option<String>,
    pub app_desc: Option<String>,
    pub app_status: String,
    pub pubkey: Option<String>,
    pub privkey: Option<String>,
    pub wx_id: Option<String>,
    pub is_wx_app: Option<bool>,
}

impl HtyApp {
    pub fn find_by_id(id_app: &String, conn: &mut PgConnection) -> anyhow::Result<HtyApp> {
        match hty_apps::table
            .filter(hty_apps::app_id.eq(id_app))
            .select(hty_apps::all_columns)
            .first::<HtyApp>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                error!("find_by_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_pubkey_by_id(app_id: &String, conn: &mut PgConnection) -> anyhow::Result<String> {
        match hty_apps::table
            .filter(hty_apps::app_id.eq(app_id))
            .select(hty_apps::all_columns)
            .first::<HtyApp>(conn)
        {
            Ok(app) => {
                let pubkey = app.pubkey.clone()
                    .ok_or_else(|| anyhow::anyhow!("pubkey is not set for app"))?;
                debug!(
                    "find_pubkey_by_id -> find pubkey : {}",
                    pubkey
                );
                Ok(pubkey)
            }
            Err(e) => {
                error!("find_pubkey_by_id -> err -> {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string())
                }))
            }
        }
    }

    pub fn get_encrypt_app_id(pubkey: &String, conn: &mut PgConnection) -> anyhow::Result<String> {
        match hty_apps::table
            .filter(hty_apps::pubkey.eq(pubkey))
            .select(hty_apps::all_columns)
            .first::<HtyApp>(conn)
        {
            Ok(app) => {
                let privkey = app.privkey
                    .ok_or_else(|| anyhow::anyhow!("privkey is not set for app"))?;
                match encrypt_text_with_private_key(privkey, app.app_id.clone()) {
                    Ok(encrypt_app_id) => {
                        debug!(
                            "get_encrypt_app_id -> encrypt app id : {}",
                            encrypt_app_id.clone()
                        );
                        Ok(encrypt_app_id)
                    }
                    Err(e) => {
                        error!("get_encrypt_app_id -> encrypt app id error : {}", e);
                        Err(anyhow!(HtyErr {
                            code: HtyErrCode::DbErr,
                            reason: Some(e.to_string())
                        }))
                    }
                }
            }
            Err(e) => {
                error!("get_encrypt_app_id -> err -> {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string())
                }))
            }
        }
    }

    pub fn find_by_domain(domain: &String, conn: &mut PgConnection) -> anyhow::Result<HtyApp> {
        match hty_apps::table
            .filter(hty_apps::domain.eq(domain))
            .select(hty_apps::all_columns)
            .first::<HtyApp>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => {
                debug!("hty_app -> find_by_domain: {:?} / err -> {:?}", domain, e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                }))
            }
        }
    }

    pub fn find_linked_roles(&self, conn: &mut PgConnection) -> anyhow::Result<Vec<HtyRole>> {
        let res = AppRole::belonging_to(self)
            .inner_join(hty_roles::table)
            .select(hty_roles::all_columns)
            .load::<HtyRole>(conn)?;
        Ok(res)
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_apps::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_app_id(in_app_id: &str, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::hty_apps::dsl::*;
        match diesel::delete(hty_apps.find(in_app_id)).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn create(in_hty_app: &HtyApp, conn: &mut PgConnection) -> anyhow::Result<HtyApp> {
        use crate::schema::hty_apps::dsl::*;
        match insert_into(hty_apps).values(in_hty_app).get_result(conn) {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_apps(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyApp>> {
        use crate::schema::hty_apps::dsl::*;
        match hty_apps.get_results(conn) {
            Ok(sections) => Ok(sections),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete(in_openid: &str, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::user_app_info::columns::*;
        use crate::schema::user_app_info::dsl::*;
        match diesel::delete(user_app_info.filter(openid.eq(in_openid))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_id(id_app: &String, conn: &mut PgConnection) -> anyhow::Result<bool> {
        use crate::schema::hty_apps::dsl::*;
        let result = select(exists(hty_apps.filter(app_id.eq(id_app))))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn update(in_hty_app: &HtyApp, conn: &mut PgConnection) -> anyhow::Result<HtyApp> {
        let result = update(hty_apps::table)
            .filter(hty_apps::app_id.eq(in_hty_app.clone().app_id))
            .set(in_hty_app)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn all_user_infos(&self, conn: &mut PgConnection) -> anyhow::Result<Vec<UserAppInfo>> {
        UserAppInfo::find_all_by_app_id(&self.app_id, conn)
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
QueryableByName,
)]
#[diesel(table_name = hty_resources)]
#[diesel(primary_key(hty_resource_id))]
#[diesel(belongs_to(HtyUser, foreign_key = created_by))]
pub struct HtyResource {
    pub filename: Option<String>,
    pub app_id: String,
    pub hty_resource_id: String,
    pub created_at: Option<NaiveDateTime>,
    pub url: String,
    pub res_type: Option<String>,
    pub created_by: Option<String>,
    pub tasks: Option<MultiVals<CommonTask>>,
    pub compress_processed: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}


impl HtyResource {
    pub fn remote_delete_by_id(id_hty_resource: &String, sudoer: &String, host: &String) -> anyhow::Result<()> {
        let c_id = id_hty_resource.clone();
        let c_sudoer = sudoer.clone();
        let c_host = host.clone();

        debug!("remote_delete_by_id -> id: {}", id_hty_resource);

        task::spawn(async move {
            // let client = reqwest::blocking::Client::new();
            let client = reqwest::Client::new();
            let resp = client.get(format!("{}/delete_hty_resource_by_id/{}", get_uc_url(), c_id))
                .header("HtySudoerToken", c_sudoer)
                .header("HtyHost", c_host)
                .send().await;
            debug!("remote_delete_by_id -> resp: {:?}", resp);
        });

        Ok(())
    }

    pub fn create(resource: &HtyResource, conn: &mut PgConnection) -> anyhow::Result<HtyResource> {
        use crate::schema::hty_resources::dsl::*;
        insert_into(hty_resources)
            .values(resource)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn update(resource: &HtyResource, conn: &mut PgConnection) -> anyhow::Result<HtyResource> {
        debug!("HtyResource::update -> {:?}", resource);
        let result = diesel::update(hty_resources::table)
            .filter(hty_resources::hty_resource_id.eq(resource.clone().hty_resource_id))
            .set(resource)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_by_id(id_hty_resource: &String, conn: &mut PgConnection) -> anyhow::Result<()> {
        use crate::schema::hty_resources::dsl::*;
        debug!("HtyResource::delete_by_id -> {:?}", id_hty_resource);
        let _ = diesel::delete(hty_resources.filter(hty_resource_id.eq(id_hty_resource))).execute(conn)?;
        Ok(())
    }
    pub fn find_by_id(id: &str, conn: &mut PgConnection) -> anyhow::Result<HtyResource> {
        use crate::schema::hty_resources::dsl::*;
        match hty_resources.find(id).get_result::<HtyResource>(conn) {
            Ok(role) => Ok(role),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        let result = diesel::delete(hty_resources::table)
            .execute(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn strict_from(req: ReqHtyResource) -> anyhow::Result<HtyResource> {
        let req_app_id = req.app_id.ok_or(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(String::from("empty app id")),
        });

        let req_url = req.url.ok_or(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(String::from("empty resource url")),
        });

        let req_created_at = req.created_at.unwrap_or(current_local_datetime());

        let req_created_by = req.created_by.ok_or(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(String::from("empty resource created by")),
        });

        if req_url.is_err() || req_url.is_err() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some(String::from("empty resource url or appid or created by")),
            }));
        }

        let req_hty_resource_id = req.hty_resource_id.ok_or(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(String::from("empty hty_resource_id")),
        });

        Ok(Self {
            filename: req.filename,
            app_id: req_app_id?,
            hty_resource_id: req_hty_resource_id?,
            created_at: Some(req_created_at),
            url: req_url?,
            res_type: req.res_type,
            created_by: Some(req_created_by?),
            tasks: req.tasks.clone(),
            compress_processed: req.compress_processed.clone(),
            updated_at: req.updated_at.clone(),
            updated_by: req.updated_by.clone(),
        })
    }

    // pub fn find_all_by_task_id(
    //     task_id: &str,
    //     conn: &mut PgConnection,
    // ) -> anyhow::Result<Vec<HtyResource>> {
    //     match hty_resources::table
    //         .filter(hty_resources::task_id.eq(task_id.to_string()))
    //         .load::<HtyResource>(conn)
    //     {
    //         Ok(hty_resources) => Ok(hty_resources),
    //         Err(e) => Err(anyhow!(HtyErr {
    //             code: HtyErrCode::DbErr,
    //             reason: Some(e.to_string()),
    //         })),
    //     }
    // }

    pub fn find_all_by_task_id_jsonb(
        task_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyResource>> {
        let q = format!("select * from hty_resources where jsonb_path_exists(tasks, '$.vals[*].task_id ? (@ == \"{}\")')", task_id).to_string();
        let res = sql_query(q.clone()).load(conn)?;
        Ok(res)
    }

    pub fn convert_to_req_hty_resources(hty_resources: &Vec<HtyResource>) -> Vec<ReqHtyResource> {
        let mut out_hty_resource_vs = vec![];

        for hty_resource in hty_resources.iter() {
            out_hty_resource_vs.push(hty_resource.to_req().clone());
        }

        out_hty_resource_vs
    }

    pub fn to_req(&self) -> ReqHtyResource {
        ReqHtyResource {
            app_id: Some(self.app_id.clone()),
            created_at: self.created_at.clone(),
            created_by: self.created_by.clone(),
            filename: self.filename.clone(),
            hty_resource_id: Some(self.hty_resource_id.clone()),
            res_type: self.res_type.clone(),
            url: Some(self.url.clone()),
            tasks: self.tasks.clone(),
            compress_processed: self.compress_processed.clone(),
            updated_at: self.updated_at.clone(),
            updated_by: self.updated_by.clone(),
        }
    }
}

// created_by, res_type need confirm and update
/*impl From<ReqHtyResource> for HtyResource {
    fn from(req: ReqHtyResource) -> HtyResource {
        let req_app_id = req.app_id.ok_or(HtyErr{
            code:HtyErrCode::WebErr,
            reason:Some(String::from("empty app id")),
        });
        let req_url = req.url.ok_or(HtyErr{
            code:HtyErrCode::WebErr,
            reason:Some(String::from("empty resource url")),
        });

        Self {
            filename: req.filename,
            app_id: req_app_id?,
            hty_resource_id: uuid(),
            created_at: Some(current_local_datetime()),
            url: req_url?,
            res_type: 1,
            created_by: None,
        }

    }
}*/

#[derive(
Identifiable,
PartialEq,
// Associations,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_roles)]
#[diesel(primary_key(hty_role_id))]
pub struct HtyRole {
    pub hty_role_id: String,
    pub role_key: String,
    pub role_desc: Option<String>,
    pub role_status: String,
    pub style: Option<String>,
    pub role_name: Option<String>,
}

impl HtyRole {
    pub fn to_req(&self) -> ReqHtyRole {
        ReqHtyRole {
            hty_role_id: Some(self.hty_role_id.clone()),
            user_app_info_id: None,
            app_ids: None,
            role_key: Some(self.role_key.clone()),
            role_desc: self.role_desc.clone(),
            role_status: Some(self.role_status.clone()),
            labels: None, // todo: fill in fields
            actions: None,
            style: self.style.clone(),
            role_name: self.role_name.clone(),
        }
    }
    pub fn create(role: &HtyRole, conn: &mut PgConnection) -> anyhow::Result<HtyRole> {
        use crate::schema::hty_roles::dsl::*;
        insert_into(hty_roles)
            .values(role)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn find_by_id(rid: &str, conn: &mut PgConnection) -> anyhow::Result<HtyRole> {
        use crate::schema::hty_roles::dsl::*;
        match hty_roles.find(rid).get_result::<HtyRole>(conn) {
            Ok(role) => Ok(role),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_key(name: &str, conn: &mut PgConnection) -> anyhow::Result<HtyRole> {
        use crate::schema::hty_roles::dsl::*;
        match hty_roles
            .filter(role_key.eq(name.to_string()))
            .get_result::<HtyRole>(conn)
        {
            Ok(role) => Ok(role),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyRole>> {
        use crate::schema::hty_roles::dsl::*;
        match hty_roles.load::<HtyRole>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(in_hty_role: &HtyRole, conn: &mut PgConnection) -> anyhow::Result<HtyRole> {
        let result = update(hty_roles::table)
            .filter(hty_roles::hty_role_id.eq(in_hty_role.clone().hty_role_id))
            .set(in_hty_role)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_roles::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_linked_action_and_label(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(Vec<HtyAction>, Vec<HtyLabel>)> {
        let action_list = RoleAction::belonging_to(self)
            .inner_join(hty_actions::table)
            .select(hty_actions::all_columns)
            .load::<HtyAction>(conn)?;

        let label_list = RoleLabel::belonging_to(self)
            .inner_join(hty_labels::table)
            .select(hty_labels::all_columns)
            .load::<HtyLabel>(conn)?;

        Ok((action_list, label_list))
    }

    pub fn find_linked_user_app_info(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<UserAppInfo>> {
        let res = UserInfoRole::belonging_to(self)
            .inner_join(user_app_info::table)
            .select(user_app_info::all_columns)
            .load::<UserAppInfo>(conn)?;
        Ok(res)
    }

    pub fn verify_exist_by_id(
        id_hty_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::hty_roles::dsl::*;
        debug!("verify_hty_role exist_by_id -> {:?}", id_hty_role);
        select(exists(hty_roles.filter(hty_role_id.eq(id_hty_role))))
            .get_result(conn)
            .map_err(|e| {
                error!("verify_exist_by_hty_role_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = user_info_roles)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(UserAppInfo, foreign_key = user_info_id))]
#[diesel(belongs_to(HtyRole, foreign_key = role_id))]
pub struct UserInfoRole {
    pub the_id: String,
    pub user_info_id: String,
    pub role_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqUserInfoRole {
    pub the_id: Option<String>,
    pub user_info_id: Option<String>,
    pub role_id: Option<String>,
}

impl UserInfoRole {
    pub fn create(entry: &UserInfoRole, conn: &mut PgConnection) -> anyhow::Result<UserInfoRole> {
        use crate::schema::user_info_roles::dsl::*;
        insert_into(user_info_roles)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(user_info_roles::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_user_info_id_and_role_id(
        id_user_info: &String,
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::user_info_roles::dsl::*;
        select(exists(user_info_roles.filter(
            user_info_id.eq(id_user_info).and(role_id.eq(id_role)),
        )))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }

    pub fn get_all_roles_by_user_info_id(
        id_user_info: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<String>> {
        use crate::schema::user_info_roles::columns::*;
        use crate::schema::user_info_roles::dsl::*;

        match user_info_roles
            .filter(user_info_id.eq(id_user_info))
            .select(role_id)
            .get_results::<String>(conn)
        {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_id(in_id: &String, conn: &mut PgConnection) -> bool {
        use crate::schema::user_info_roles::dsl::*;
        match diesel::delete(user_info_roles.find(in_id)).execute(conn) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn delete_by_user_info_id(
        id_user_info: &String,
        conn: &mut PgConnection,
    ) -> Result<usize, HtyErr> {
        use crate::schema::user_info_roles::dsl::*;
        match diesel::delete(user_info_roles.filter(user_info_id.eq(id_user_info))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            }),
        }
    }

    pub fn find_by_role_id_and_user_info_id(
        id_role: &String,
        id_user_info: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<UserInfoRole> {
        use crate::schema::user_info_roles::columns::*;
        use crate::schema::user_info_roles::dsl::*;

        match user_info_roles
            .filter(user_info_id.eq(id_user_info))
            .filter(role_id.eq_all(id_role))
            .first::<UserInfoRole>(conn)
        {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_role_id_and_user_info_id(
        id_role: &String,
        id_user_info: &String,
        conn: &mut PgConnection,
    ) -> Result<usize, HtyErr> {
        use crate::schema::user_info_roles::dsl::*;
        match diesel::delete(
            user_info_roles.filter(user_info_id.eq(id_user_info).and(role_id.eq(id_role))),
        )
            .execute(conn)
        {
            Ok(num) => Ok(num),
            Err(e) => Err(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            }),
        }
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = roles_labels)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(HtyRole, foreign_key = role_id))]
#[diesel(belongs_to(HtyLabel, foreign_key = label_id))]
pub struct RoleLabel {
    pub the_id: String,
    pub role_id: String,
    pub label_id: String,
}

impl RoleLabel {
    pub fn create(entry: &RoleLabel, conn: &mut PgConnection) -> Result<RoleLabel, HtyErr> {
        use crate::schema::roles_labels::dsl::*;
        insert_into(roles_labels)
            .values(entry)
            .get_result(conn)
            .map_err(|e| HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(roles_labels::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_role_id(id_role: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::roles_labels::dsl::*;
        match diesel::delete(roles_labels.filter(role_id.eq(id_role))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_label_id(id_label: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::roles_labels::dsl::*;
        match diesel::delete(roles_labels.filter(label_id.eq(id_label))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_role_id_and_label_id(
        id_role2: &String,
        id_label: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::roles_labels::dsl::*;
        select(exists(
            roles_labels.filter(role_id.eq(id_role2).and(label_id.eq(id_label))),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = roles_actions)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(HtyRole, foreign_key = role_id))]
#[diesel(belongs_to(HtyAction, foreign_key = action_id))]
pub struct RoleAction {
    pub the_id: String,
    pub role_id: String,
    pub action_id: String,
}

impl RoleAction {
    pub fn create(entry: &RoleAction, conn: &mut PgConnection) -> anyhow::Result<RoleAction> {
        use crate::schema::roles_actions::dsl::*;
        insert_into(roles_actions)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(roles_actions::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_role_id_and_action_id(
        id_role: &String,
        id_action: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::roles_actions::dsl::*;
        select(exists(
            roles_actions.filter(role_id.eq(id_role).and(action_id.eq(id_action))),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }

    pub fn delete_by_role_id(id_role: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::roles_actions::dsl::*;
        match diesel::delete(roles_actions.filter(role_id.eq(id_role))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_action_id(
        id_action: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<usize> {
        use crate::schema::roles_actions::dsl::*;
        match diesel::delete(roles_actions.filter(action_id.eq(id_action))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = apps_roles)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
#[diesel(belongs_to(HtyRole, foreign_key = role_id))]
pub struct AppRole {
    pub the_id: String,
    pub app_id: String,
    pub role_id: String,
}

impl AppRole {
    pub fn create(entry: &AppRole, conn: &mut PgConnection) -> anyhow::Result<AppRole> {
        use crate::schema::apps_roles::dsl::*;
        insert_into(apps_roles)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn delete_all_by_app_id(id_app: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::apps_roles::dsl::*;
        match diesel::delete(apps_roles.filter(app_id.eq(id_app))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(apps_roles::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_role_id(id_role: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::apps_roles::dsl::*;
        match diesel::delete(apps_roles.filter(role_id.eq(id_role))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_app_id_and_role_id(
        id_app: &String,
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::apps_roles::dsl::*;
        select(exists(
            apps_roles.filter(app_id.eq(id_app).and(role_id.eq(id_role))),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }
}

#[derive(
Identifiable,
PartialEq,
Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = actions_labels)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(HtyAction, foreign_key = action_id))]
#[diesel(belongs_to(HtyLabel, foreign_key = label_id))]
pub struct ActionLabel {
    pub the_id: String,
    pub action_id: String,
    pub label_id: String,
}

impl ActionLabel {
    pub fn create(entry: &ActionLabel, conn: &mut PgConnection) -> anyhow::Result<ActionLabel> {
        use crate::schema::actions_labels::dsl::*;
        insert_into(actions_labels)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(actions_labels::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_action_id_and_label_id(
        id_action: &String,
        id_label: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::actions_labels::dsl::*;
        select(exists(
            actions_labels.filter(action_id.eq(id_action).and(label_id.eq(id_label))),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            })
    }

    pub fn delete_by_action_id(
        id_action: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<usize> {
        use crate::schema::actions_labels::dsl::*;
        match diesel::delete(actions_labels.filter(action_id.eq(id_action))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_label_id(id_label: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::actions_labels::dsl::*;
        match diesel::delete(actions_labels.filter(label_id.eq(id_label))).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_labels)]
#[diesel(primary_key(hty_label_id))]
pub struct HtyLabel {
    pub hty_label_id: String,
    pub label_name: String,
    pub label_desc: Option<String>,
    pub label_status: String,
    pub style: Option<String>,
}

impl HtyLabel {
    pub fn to_req_labels(in_labels: &Vec<HtyLabel>) -> Vec<ReqHtyLabel> {
        let mut out_labels = Vec::new();

        for in_label in in_labels {
            out_labels.push(in_label.clone().to_req());
        }

        out_labels
    }

    pub fn to_req(&self) -> ReqHtyLabel {
        ReqHtyLabel {
            hty_label_id: Some(self.hty_label_id.clone()),
            label_name: Some(self.label_name.clone()),
            label_desc: self.label_desc.clone(),
            label_status: Some(self.label_status.clone()),
            roles: None,
            actions: None,
            style: self.style.clone(),
        }
    }

    pub fn find_all_by_role_id(
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyLabel>> {
        let the_role = HtyRole::find_by_id(id_role, conn)?;
        let the_labels = RoleLabel::belonging_to(&the_role)
            .inner_join(hty_labels::table)
            .select(hty_labels::all_columns)
            .load::<HtyLabel>(conn)?;
        Ok(the_labels)
    }

    pub fn create(entry: &HtyLabel, conn: &mut PgConnection) -> anyhow::Result<HtyLabel> {
        use crate::schema::hty_labels::dsl::*;
        insert_into(hty_labels)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn update(in_hty_label: &HtyLabel, conn: &mut PgConnection) -> anyhow::Result<HtyLabel> {
        let result = update(hty_labels::table)
            .filter(hty_labels::hty_label_id.eq(in_hty_label.clone().hty_label_id))
            .set(in_hty_label)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyLabel>> {
        use crate::schema::hty_labels::dsl::*;
        match hty_labels.load::<HtyLabel>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_id(lid: &str, conn: &mut PgConnection) -> anyhow::Result<HtyLabel> {
        use crate::schema::hty_labels::dsl::*;
        match hty_labels.find(lid).get_result::<HtyLabel>(conn) {
            Ok(role) => Ok(role),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_linked_role_and_action(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(Vec<HtyRole>, Vec<HtyAction>)> {
        let role_list = RoleLabel::belonging_to(self)
            .inner_join(hty_roles::table)
            .select(hty_roles::all_columns)
            .load::<HtyRole>(conn)?;

        let action_list = ActionLabel::belonging_to(self)
            .inner_join(hty_actions::table)
            .select(hty_actions::all_columns)
            .load::<HtyAction>(conn)?;

        Ok((role_list, action_list))
    }

    pub fn to_req_label(&self) -> ReqHtyLabel {
        let self_copy = self.clone();
        ReqHtyLabel {
            hty_label_id: Some(self_copy.hty_label_id),
            label_name: Some(self_copy.label_name),
            label_desc: self_copy.label_desc,
            label_status: Some(self_copy.label_status),
            roles: None,
            actions: None,
            style: self_copy.style,
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_labels::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_id(
        id_hty_label: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::hty_labels::dsl::*;
        debug!("verify_hty_label_exist_by_id -> {:?}", id_hty_label);
        select(exists(hty_labels.filter(hty_label_id.eq(id_hty_label))))
            .get_result(conn)
            .map_err(|e| {
                error!("verify_exist_by_hty_label_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }
}

#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_actions)]
#[diesel(primary_key(hty_action_id))]
pub struct HtyAction {
    pub hty_action_id: String,
    pub action_name: String,
    pub action_desc: Option<String>,
    pub action_status: String,
}

impl HtyAction {
    pub fn create(entry: &HtyAction, conn: &mut PgConnection) -> anyhow::Result<HtyAction> {
        use crate::schema::hty_actions::dsl::*;
        insert_into(hty_actions)
            .values(entry)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            })
    }

    pub fn update(in_hty_action: &HtyAction, conn: &mut PgConnection) -> anyhow::Result<HtyAction> {
        let result = update(hty_actions::table)
            .filter(hty_actions::hty_action_id.eq(in_hty_action.clone().hty_action_id))
            .set(in_hty_action)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyAction>> {
        use crate::schema::hty_actions::dsl::*;
        match hty_actions.load::<HtyAction>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_id(aid: &str, conn: &mut PgConnection) -> anyhow::Result<HtyAction> {
        use crate::schema::hty_actions::dsl::*;
        match hty_actions.find(aid).get_result::<HtyAction>(conn) {
            Ok(role) => Ok(role),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_linked_role_and_label(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(Vec<HtyRole>, Vec<HtyLabel>)> {
        let res_labels = self.find_linked_label(conn)?;

        let res_roles = RoleAction::belonging_to(self)
            .inner_join(hty_roles::table)
            .select(hty_roles::all_columns)
            .load::<HtyRole>(conn)?;

        Ok((res_roles, res_labels))
    }

    pub fn find_linked_label(&self, conn: &mut PgConnection) -> anyhow::Result<Vec<HtyLabel>> {
        let res = ActionLabel::belonging_to(self)
            .inner_join(hty_labels::table)
            .select(hty_labels::all_columns)
            .load::<HtyLabel>(conn)?;
        Ok(res)
    }

    pub fn to_req_action(&self, conn: &mut PgConnection) -> anyhow::Result<ReqHtyAction> {
        let labels = self.find_linked_label(conn)?;
        let linked_labels = labels.iter().map(|label| label.to_req_label()).collect();
        Ok(ReqHtyAction {
            hty_action_id: Some(self.clone().hty_action_id),
            action_name: Some(self.clone().action_name),
            action_desc: self.clone().action_desc,
            action_status: Some(self.clone().action_status),
            roles: None,
            labels: Some(linked_labels),
        })
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_actions::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_id(id_action: &String, conn: &mut PgConnection) -> anyhow::Result<bool> {
        use crate::schema::hty_actions::dsl::*;
        let result = select(exists(hty_actions.filter(hty_action_id.eq(id_action))))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }
}

#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
// Associations,
)]
#[diesel(table_name = hty_tags)]
// #[diesel(belongs_to(HtyTagRef, foreign_key = tag_id))]
#[diesel(primary_key(tag_id))]
pub struct HtyTag {
    pub tag_id: String,
    pub tag_name: String,
    pub tag_desc: Option<String>,
    pub style: Option<String>,
}

impl HtyTag {
    pub fn to_req(&self) -> ReqHtyTag {
        ReqHtyTag {
            tag_id: Some(self.tag_id.clone()),
            tag_name: Some(self.tag_name.clone()),
            tag_desc: self.tag_desc.clone(),
            style: self.style.clone(),
            refs: None,
        }
    }

    pub fn create(in_tag: &HtyTag, conn: &mut PgConnection) -> anyhow::Result<HtyTag> {
        use crate::schema::hty_tags::dsl::*;
        match insert_into(hty_tags).values(in_tag).get_result(conn) {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(in_tag: &HtyTag, conn: &mut PgConnection) -> anyhow::Result<HtyTag> {
        let result = update(hty_tags::table)
            .filter(hty_tags::tag_id.eq(in_tag.clone().tag_id))
            .set(in_tag)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });

        result
    }

    pub fn find_by_tag_id(id_to_find: &String, conn: &mut PgConnection) -> anyhow::Result<HtyTag> {
        match hty_tags::table
            .filter(hty_tags::tag_id.eq(id_to_find))
            .select(hty_tags::all_columns)
            .first::<HtyTag>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_all_by_ref_id(id_to_find: &String, conn: &mut PgConnection) -> anyhow::Result<Vec<HtyTag>> {
        debug!("find_all_by_ref_id -> ref_id: {:?}", id_to_find);

        let tag_refs = HtyTagRef::find_all_by_ref_id(id_to_find, conn)?;
        debug!("find_all_by_ref_id -> tag_refs: {:?}", tag_refs);

        // let res: Vec<HtyTag> = HtyTag::belonging_to(&tag_refs).load::<HtyTag>(conn)?;

        let mut res: Vec<HtyTag> = vec![];

        for tag_ref in tag_refs.iter() {
            res.push(HtyTag::find_by_tag_id(&tag_ref.hty_tag_id, conn)?);
        }

        debug!("find_all_by_ref_id -> tags: {:?}", res);

        Ok(res)
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyTag>> {
        use crate::schema::hty_tags::dsl::*;
        match hty_tags.load::<HtyTag>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_tags::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

// todo: Use enum type for ref_type.
#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
Associations,
)]
#[diesel(table_name = hty_tag_refs)]
#[diesel(primary_key(the_id))]
#[diesel(belongs_to(HtyTag, foreign_key = hty_tag_id))]
pub struct HtyTagRef {
    pub the_id: String,
    pub hty_tag_id: String,
    pub ref_id: String,
    pub ref_type: String,
    pub meta: Option<CommonMeta>,
}

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
#[allow(non_snake_case)]
pub struct MetaUserAppInfo {
    pub nickName: Option<String>,
    pub avatarUrl: Option<String>,
    pub gender: Option<u32>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub mother_job: Option<String>,
    pub father_job: Option<String>,
}

impl_jsonb_boilerplate!(MetaUserAppInfo);

impl HtyTagRef {
    pub fn all_to_reqs(in_all: &Vec<HtyTagRef>) -> Vec<ReqHtyTagRef> {
        let mut resp_req_tag_ref = Vec::new();
        for item in in_all {
            let c_item = item.clone();
            let req_comment = ReqHtyTagRef {
                tag_ref_id: Some(c_item.the_id.clone()),
                hty_tag_id: Some(c_item.hty_tag_id.clone()),
                ref_id: Some(c_item.ref_id.clone()),
                ref_type: Some(c_item.ref_type.clone()),
                meta: c_item.meta.clone(),
                tag: None, // todo: @buddy 这里需要数据库取出
            };
            resp_req_tag_ref.push(req_comment);
        }
        resp_req_tag_ref
    }

    pub fn to_req(&self) -> ReqHtyTagRef {
        ReqHtyTagRef {
            tag_ref_id: Some(self.the_id.clone()),
            hty_tag_id: Some(self.hty_tag_id.clone()),
            ref_id: Some(self.ref_id.clone()),
            ref_type: Some(self.ref_type.clone()),
            meta: self.meta.clone(),
            tag: None, // todo: @buddy 这里需要数据库取出
        }
    }

    pub fn create(in_tag_ref: &HtyTagRef, conn: &mut PgConnection) -> anyhow::Result<HtyTagRef> {
        use crate::schema::hty_tag_refs::dsl::*;
        match insert_into(hty_tag_refs)
            .values(in_tag_ref)
            .get_result(conn)
        {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_by_ref_id(
        id_to_find: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTagRef>> {
        match hty_tag_refs::table
            .filter(hty_tag_refs::ref_id.eq(id_to_find.to_string()))
            .load::<HtyTagRef>(conn)
        {
            Ok(refs) => Ok(refs),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all_by_ref_id(
        id_to_find: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<i32> {
        use crate::schema::hty_tag_refs::dsl::*;

        let res =
            diesel::delete(hty_tag_refs.filter(ref_id.eq(id_to_find.to_string()))).execute(conn)?;

        Ok(res as i32)
    }

    pub fn find_all_by_tag_id(
        id_tag: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTagRef>> {
        debug!("find_all_by_tag_id -> id_tag: {:?}", id_tag);

        match hty_tag_refs::table
            .filter(hty_tag_refs::hty_tag_id.eq(id_tag.to_string()))
            .load::<HtyTagRef>(conn)
        {
            Ok(refs) => Ok(refs),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn verify_exist_by_ref_id_and_tag_id(
        id_ref: &String,
        id_tag: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<bool> {
        debug!(
            "verify_exist_by_ref_id_and_tag_id -> id_ref: {:?} / id_tag: {:?}",
            id_ref, id_tag
        );
        if id_ref.is_empty() || id_tag.is_empty() {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(
                    "verify_exist_by_ref_id_and_tag_id -> `id_tag` or `id_ref` can't be NULL!"
                        .to_string()
                )
            }));
        }

        use crate::schema::hty_tag_refs::dsl::*;
        let result = select(exists(
            hty_tag_refs.filter(hty_tag_id.eq(id_tag).and(ref_id.eq(id_ref))),
        ))
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
            });
        result
    }

    pub fn delete_by_id(id_to_delete: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::hty_tag_refs::dsl::*;
        match diesel::delete(hty_tag_refs.find(id_to_delete)).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_tag_refs::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

// 发送方域名拿到from_app -> 查找需要推送的目标app（一般是公众号）
#[derive(
Identifiable,
PartialEq,
// Associations,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
AsChangeset,
)]
#[diesel(table_name = app_from_to)]
#[diesel(primary_key(id))]
// #[diesel(belongs_to(FromHtyApp, foreign_key = "from_app_id"))]
// #[diesel(belongs_to(ToHtyApp, foreign_key = "to_app_id"))]
pub struct AppFromTo {
    pub id: String,
    pub from_app_id: String,
    pub to_app_id: String,
    pub is_enabled: bool,
}

impl AppFromTo {
    pub fn find_by_from_and_to_id(
        id_from: &String,
        id_to: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<AppFromTo> {
        match app_from_to::table
            .filter(app_from_to::from_app_id.eq(id_from))
            .filter(app_from_to::to_app_id.eq(id_to))
            .first(conn)
        {
            Ok(all) => Ok(all),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::NotFoundErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn all_to_reqs(in_all: &Vec<AppFromTo>) -> Vec<ReqAppFromTo> {
        let mut resp_req_appfromto = Vec::new();
        for item in in_all {
            let c_item = item.clone();
            let req_comment = ReqAppFromTo {
                id: Some(c_item.id),
                from_app_id: Some(c_item.from_app_id),
                to_app_id: Some(c_item.to_app_id),
                is_enabled: Some(c_item.is_enabled),
            };
            resp_req_appfromto.push(req_comment);
        }
        resp_req_appfromto
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<AppFromTo>> {
        use crate::schema::app_from_to::dsl::*;
        let result = app_from_to.get_results(conn).map_err(|e| {
            anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })
        });
        result
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(app_from_to::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_by_id(del_id: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::app_from_to::dsl::*;
        match diesel::delete(app_from_to.find(del_id)).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_id(id_to_find: &String, conn: &mut PgConnection) -> anyhow::Result<AppFromTo> {
        use crate::schema::app_from_to::dsl::*;
        match app_from_to.find(id_to_find).get_result::<AppFromTo>(conn) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(in_item: &AppFromTo, conn: &mut PgConnection) -> anyhow::Result<AppFromTo> {
        let result = update(app_from_to::table)
            .filter(app_from_to::id.eq(in_item.clone().id))
            .set(in_item)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });

        result
    }

    pub fn disable_by_id(
        to_disable_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<AppFromTo> {
        let mut to_disable = AppFromTo::find_by_id(to_disable_id, conn)?;
        to_disable.is_enabled = false;
        AppFromTo::update(&to_disable, conn)
    }

    pub fn enable_by_id(
        to_enable_id: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<AppFromTo> {
        let mut to_enable = AppFromTo::find_by_id(to_enable_id, conn)?;
        to_enable.is_enabled = true;
        AppFromTo::update(&to_enable, conn)
    }

    pub fn create(in_from_to: &AppFromTo, conn: &mut PgConnection) -> anyhow::Result<AppFromTo> {
        use crate::schema::app_from_to::dsl::*;
        match insert_into(app_from_to).values(in_from_to).get_result(conn) {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_active_to_apps_by_from_app(
        id_from_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<AppFromTo>> {
        match app_from_to::table
            .filter(app_from_to::from_app_id.eq(id_from_app))
            .filter(app_from_to::is_enabled.eq(true))
            .load::<AppFromTo>(conn)
        {
            Ok(all) => Ok(all),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
Associations,
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_tongzhi)]
#[diesel(primary_key(tongzhi_id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
#[diesel(belongs_to(HtyRole, foreign_key = role_id))]
pub struct HtyTongzhi {
    pub tongzhi_id: String,
    pub app_id: String,
    pub tongzhi_type: String,
    pub tongzhi_status: String,
    pub send_from: Option<String>,
    // 目前diesel不支持两个字段外键指向到同一张表，
    pub send_to: String,
    // 所以这两个字段就先不在diesel层面指定对hty_users的ref关系.
    pub created_at: NaiveDateTime,
    pub content: Option<CommonTongzhiContent>,
    pub meta: Option<TongzhiMeta>,
    pub role_id: Option<String>,
    pub push_info: Option<PushInfo>,
}


impl HtyTongzhi {
    pub fn to_req(&self) -> ReqHtyTongzhi {
        ReqHtyTongzhi {
            tongzhi_id: Some(self.tongzhi_id.clone()),
            app_id: Some(self.app_id.clone()),
            tongzhi_type: Some(self.tongzhi_type.clone()),
            tongzhi_status: Some(self.tongzhi_status.clone()),
            send_from: self.send_from.clone(),
            send_to: Some(self.send_to.clone()),
            created_at: Some(self.created_at.clone()),
            content: self.content.clone(),
            meta: self.meta.clone(),
            role_id: self.role_id.clone(),
            push_info: self.push_info.clone(),
        }
    }


    pub fn count_unread_tongzhis_by_user_id_and_role_id(
        id_user: &String,
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<i32> {
        // todo: rewrite with diesel query, no need SQL
        let q = format!("select count(*) as result from hty_tongzhi where tongzhi_status='{}' and send_to='{}' and role_id='{}';", UNREAD, id_user, id_role).to_string();

        debug!(
            "SQL - {}",
            format!("count_unread_tongzhis_by_user_id_and_role_id : {:?}", q)
        );

        let res = sql_query(q.clone()).get_result::<CountResult>(conn)?.result;

        Ok(res as i32)
    }

    pub fn create(tongzhi: &HtyTongzhi, conn: &mut PgConnection) -> anyhow::Result<HtyTongzhi> {
        use crate::schema::hty_tongzhi::dsl::*;
        match insert_into(hty_tongzhi).values(tongzhi).get_result(conn) {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(in_tongzhi: &HtyTongzhi, conn: &mut PgConnection) -> anyhow::Result<HtyTongzhi> {
        let result = update(hty_tongzhi::table)
            .filter(hty_tongzhi::tongzhi_id.eq(in_tongzhi.clone().tongzhi_id))
            .set(in_tongzhi)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn find_by_id(id_tongzhi: &String, conn: &mut PgConnection) -> anyhow::Result<HtyTongzhi> {
        match hty_tongzhi::table
            .filter(hty_tongzhi::tongzhi_id.eq(id_tongzhi))
            .select(hty_tongzhi::all_columns)
            .first::<HtyTongzhi>(conn)
        {
            Ok(tongzhi) => Ok(tongzhi),
            Err(e) => Err({
                error!("find_by_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_by_id2(id_tongzhi: &String, conn: &mut PgConnection) -> anyhow::Result<Option<HtyTongzhi>> {
        match hty_tongzhi::table
            .filter(hty_tongzhi::tongzhi_id.eq(id_tongzhi))
            .select(hty_tongzhi::all_columns)
            .first::<HtyTongzhi>(conn).optional()
        {
            Ok(some_tongzhi) => Ok(some_tongzhi),
            Err(e) => Err({
                error!("find_by_id2 / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_unreads_by_send_to_and_app_id(
        send_to_id: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTongzhi>> {
        use crate::schema::hty_tongzhi::dsl::*;
        match hty_tongzhi
            .filter(
                app_id
                    .eq(id_app)
                    .and(send_to.eq(send_to_id))
                    .and(tongzhi_status.eq(String::from(UNREAD))),
            )
            .load::<HtyTongzhi>(conn)
        {
            Ok(users) => Ok(users),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_tongzhis_with_page(
        page: &Option<i64>,
        page_size: &Option<i64>,
        the_tongzhi_status: &Option<String>,
        the_role_id: &Option<String>,
        hty_id: &Option<&String>,
        keyword: &Option<&String>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(Vec<HtyTongzhi>, i64, i64)> {
        use crate::schema::hty_tongzhi::columns::*;
        use htycommons::pagination::*;

        let mut q = hty_tongzhi::table.into_boxed();

        debug!("find_tongzhis_with_page -> page: {:?} / page_size: {:?} / the_tongzhi_status: {:?} / the_role_id: {:?} / hty_id: {:?}",
        page,
        page_size,
        the_tongzhi_status,
        the_role_id,
        hty_id
        );

        if let Some(hty_id_val) = hty_id.clone() {
            if keyword.is_some() {
                //后续如果有新增搜索功能再填充
                q = q
                    .filter(send_to.eq(hty_id_val.clone()));

            } else {
                q = q
                    .filter(send_to.eq(hty_id_val.clone()));

            }
        } else if keyword.is_some() {
            // todo: 这里有keyword搜索吗？
            if let Some(hty_id_val) = hty_id.clone() {
                q = q
                    .filter(send_to.eq(hty_id_val));
            }
        }

        if let Some(tongzhi_status_val) = the_tongzhi_status.clone() {
            q = q.filter(tongzhi_status.eq(tongzhi_status_val))
        }

        if let Some(role_id_val) = the_role_id.clone() {
            q = q.filter(role_id.eq(role_id_val))
        }

        q = q.order(created_at.desc());

        let (resp_tongzhis, total_page, total) = q
            // .load_with_pagination(conn, page.clone(), page_size.clone())?;
            .paginate(page.clone())
            .per_page(page_size.clone())
            .load_and_count_pages(conn)?;

        debug!(
            "find_tongzhis_with_page -> resp_tongzhis: {:?} / total_page: {:?} / total: {:?}",
            resp_tongzhis, total_page, total
        );

        Ok((resp_tongzhis, total_page, total))
    }

    pub fn delete_by_id(the_id: &String, conn: &mut PgConnection) -> anyhow::Result<HtyTongzhi> {
        use crate::schema::hty_tongzhi::dsl::*;

        let to_delete = HtyTongzhi::find_by_id(the_id, conn)?;

        match diesel::delete(hty_tongzhi.find(the_id)).execute(conn) {
            Ok(_) => Ok(to_delete),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_by_user_id_and_role_id(
        id_user: &String,
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTongzhi>> {
        match hty_tongzhi::table
            .filter(
                hty_tongzhi::send_to
                    .eq(id_user)
                    .and(hty_tongzhi::role_id.eq(id_role)),
            )
            .load::<HtyTongzhi>(conn)
        {
            Ok(all) => {
                debug!("find_all_by_user_id_and_role_id / OK -> {:?}", all);
                Ok(all)
            }
            Err(e) => {
                error!("find_all_by_user_id_and_role_id / ERR -> {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                }))
            }
        }
    }

    pub fn find_all_by_user_id_and_role_id_and_status(
        id_user: &String,
        id_role: &String,
        status: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTongzhi>> {
        match hty_tongzhi::table
            .filter(
                hty_tongzhi::send_to
                    .eq(id_user)
                    .and(hty_tongzhi::role_id.eq(id_role))
                    .and(hty_tongzhi::tongzhi_status.eq(status)),
            )
            .load::<HtyTongzhi>(conn)
        {
            Ok(all) => {
                debug!(
                    "find_all_by_user_id_and_role_id_and_status_id / OK -> {:?}",
                    all
                );
                Ok(all)
            }
            Err(e) => {
                error!(
                    "find_all_by_user_id_and_role_id_and_status_id / ERR -> {:?}",
                    e
                );
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                }))
            }
        }
    }

    pub fn clear_all_unread_tongzhis_by_user_id_and_role_id(
        id_user: &String,
        id_role: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<i32> {
        let all = HtyTongzhi::find_all_by_user_id_and_role_id(id_user, id_role, conn)?;

        for to_update in all.clone() {
            let mut to_update_copy = to_update.clone();
            to_update_copy.tongzhi_status = READ.to_string();
            let _ = HtyTongzhi::update(&to_update_copy, conn)?;
        }

        Ok(all.len() as i32)
    }

    pub fn delete_all_by_status_and_role_id_and_user_id(
        status: &String,
        id_role: &String,
        id_user: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<i32> {
        use crate::schema::hty_tongzhi::dsl::*;

        let to_delete_tongzhis =
            HtyTongzhi::find_all_by_user_id_and_role_id_and_status(id_user, id_role, status, conn)?;

        // todo: put into one transaction.
        // for to_delete in to_delete_tongzhis.clone() {
        //     let _ = diesel::delete(hty_tongzhi.find(&to_delete.tongzhi_id)).execute(conn);
        // }
        let tongzhi_ids: Vec<_> = to_delete_tongzhis.iter().map(|td| &td.tongzhi_id).collect();
        let _ = diesel::delete(hty_tongzhi.filter(tongzhi_id.eq_any(&tongzhi_ids))).execute(conn);

        Ok(to_delete_tongzhis.len() as i32)
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        let result = diesel::delete(hty_tongzhi::table)
            .execute(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }
}


#[derive(
AsChangeset,
// Associations,
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
)]
#[diesel(table_name = hty_template)]
pub struct HtyTemplate {
    pub id: String,
    pub template_key: String,
    pub created_at: NaiveDateTime,
    pub created_by: String,
    pub template_desc: Option<String>,
}

impl HtyTemplate {
    pub fn find_by_id(
        id_template: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplate> {
        match hty_template::table
            .filter(hty_template::id.eq(id_template))
            .select(hty_template::all_columns)
            .first::<HtyTemplate>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_by_key(
        key_template: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplate> {
        debug!(
            "HtyTemplate -> find_by_key -> key_template: {:?}",
            key_template
        );

        match hty_template::table
            .filter(hty_template::template_key.eq(key_template))
            .select(hty_template::all_columns)
            .first::<HtyTemplate>(conn)
        {
            Ok(app) => {
                debug!("HtyTemplate -> find_by_key -> app: {:?}", app);
                Ok(app)
            }
            Err(e) => {
                debug!("HtyTemplate -> find_by_key -> e: {:?}", e);
                Err(anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string())
                }))
            }
        }
    }

    pub fn find_all(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyTemplate>> {
        use crate::schema::hty_template::dsl::*;
        match hty_template.load::<HtyTemplate>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_linked_template_data<T: Debug + Serialize + DeserializeOwned + Clone + 'static>(
        &self,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyTemplateData<T>>> {
        let res = HtyTemplateData::<T>::belonging_to(self).load::<HtyTemplateData<T>>(conn)?;
        Ok(res)
    }

    pub fn create(
        in_template: &HtyTemplate,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplate> {
        use crate::schema::hty_template::dsl::*;
        match insert_into(hty_template)
            .values(in_template)
            .get_result(conn)
        {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(
        in_template: &HtyTemplate,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplate> {
        let result = update(hty_template::table)
            .filter(hty_template::id.eq(in_template.clone().id))
            .set(in_template)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn delete_by_id(id_template: &String, conn: &mut PgConnection) -> anyhow::Result<usize> {
        use crate::schema::hty_template::dsl::*;
        match diesel::delete(hty_template.find(id_template)).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_template::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
AsChangeset,
Associations,
Clone,
Debug,
Deserialize,
Identifiable,
Insertable,
PartialEq,
Queryable,
Serialize,
)]
#[serde(bound = "")]
#[diesel(belongs_to(HtyTemplate, foreign_key = template_id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
#[diesel(table_name = hty_template_data)]
pub struct HtyTemplateData<T: Debug + Serialize + DeserializeOwned + Clone> {
    pub id: String,
    pub app_id: String,
    pub template_id: String,
    pub template_val: Option<String>,
    pub template_text: Option<SingleVal<T>>,
    pub created_at: NaiveDateTime,
    pub created_by: String,
}

impl<T> HtyTemplateData<T>
    where
        T: Debug + Serialize + DeserializeOwned + Clone + 'static,
{
    pub fn create<U: Debug + DeserializeOwned + Serialize + Clone + 'static>(
        in_template_data: &HtyTemplateData<T>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<U>> {
        use crate::schema::hty_template_data::dsl::*;
        match insert_into(hty_template_data)
            .values(in_template_data)
            .get_result::<HtyTemplateData<U>>(conn)
        {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_with_template_key_and_app_id(
        key_template: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(HtyTemplate, HtyTemplateData<T>)> {
        debug!(
            "find_with_template_key_and_app_id -> key_template: {:?}, id_app: {:?}",
            key_template, id_app
        );

        let the_template = HtyTemplate::find_by_key(key_template, conn)?;
        debug!(
            "find_with_template_key_and_app_id -> the_template -> {:?}",
            the_template
        );

        let the_template_data = HtyTemplateData::<T>::find_with_template_id_and_app_id(
            &the_template.id.clone(),
            &id_app,
            conn,
        )?;
        Ok((the_template, the_template_data))
    }

    pub fn find_with_template_key_and_app_id_with_string(
        key_template: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<(HtyTemplate, HtyTemplateData<String>)> {
        debug!(
            "find_with_template_key_and_app_id_with_string -> key_template: {:?}, id_app: {:?}",
            key_template, id_app
        );

        let the_template = HtyTemplate::find_by_key(key_template, conn)?;
        debug!(
            "find_with_template_key_and_app_id_with_string -> the_template -> {:?}",
            the_template
        );

        let the_template_data = HtyTemplateData::<T>::find_with_template_id_and_app_id_with_string(
            &the_template.id.clone(),
            &id_app,
            conn,
        )?;
        Ok((the_template, the_template_data))
    }

    pub fn find_with_template_id_and_app_id(
        id_template: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<T>> {
        debug!(
            "find_with_template_id_and_app_id -> id_template: {:?}, id_app: {:?}",
            id_template, id_app
        );

        match hty_template_data::table
            .filter(
                hty_template_data::template_id
                    .eq(id_template)
                    .and(hty_template_data::app_id.eq(id_app)),
            )
            .select(hty_template_data::all_columns)
            .first::<HtyTemplateData<String>>(conn)
        {
            Ok(data) => {
                let text = data.template_text
                    .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
                    .val
                    .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;
                debug!(
                    "find_with_template_id_and_app_id -> template_text: {:?}",
                    text
                );

                let json_text;

                json_text = serde_json::from_str::<T>(text.as_str())?;

                debug!(
                    "find_with_template_id_and_app_id -> json_text: {:?}",
                    json_text
                );

                let resp_data = HtyTemplateData {
                    id: data.id.clone(),
                    app_id: data.app_id.clone(),
                    template_id: data.template_id.clone(),
                    template_val: data.template_val.clone(),
                    template_text: Some(SingleVal {
                        val: Some(json_text.clone()),
                    }),
                    created_at: data.created_at.clone(),
                    created_by: data.created_by.clone(),
                };

                debug!("find_with_template_id_and_app_id -> app: {:?}", resp_data);
                Ok(resp_data)
            }
            Err(e) => {
                debug!("find_with_template_id_and_app_id -> e: {:?}", e);
                Err({
                    anyhow!(HtyErr {
                        code: HtyErrCode::InternalErr,
                        reason: Some(e.to_string())
                    })
                })
            }
        }
    }

    pub fn find_with_template_id_and_app_id_with_string(
        id_template: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<String>> {
        debug!(
            "find_with_template_id_and_app_id_with_string -> id_template: {:?}, id_app: {:?}",
            id_template, id_app
        );

        match hty_template_data::table
            .filter(
                hty_template_data::template_id
                    .eq(id_template)
                    .and(hty_template_data::app_id.eq(id_app)),
            )
            .select(hty_template_data::all_columns)
            .first::<HtyTemplateData<String>>(conn)
        {
            Ok(data) => {
                let text = data.template_text
                    .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
                    .val
                    .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;
                debug!(
                    "find_with_template_id_and_app_id_with_string -> template_text: {:?}",
                    text
                );

                let json_text;

                json_text = text;

                debug!(
                    "find_with_template_id_and_app_id_with_string -> json_text: {:?}",
                    json_text
                );

                let resp_data = HtyTemplateData {
                    id: data.id.clone(),
                    app_id: data.app_id.clone(),
                    template_id: data.template_id.clone(),
                    template_val: data.template_val.clone(),
                    template_text: Some(SingleVal {
                        val: Some(json_text.clone()),
                    }),
                    created_at: data.created_at.clone(),
                    created_by: data.created_by.clone(),
                };

                debug!(
                    "find_with_template_id_and_app_id_with_string -> app: {:?}",
                    resp_data
                );
                Ok(resp_data)
            }
            Err(e) => {
                debug!("find_with_template_id_and_app_id_with_string -> e: {:?}", e);
                Err({
                    anyhow!(HtyErr {
                        code: HtyErrCode::InternalErr,
                        reason: Some(e.to_string())
                    })
                })
            }
        }
    }

    pub fn update(
        in_template_data: &HtyTemplateData<T>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<T>> {
        let result = update(hty_template_data::table)
            .filter(hty_template_data::id.eq(&in_template_data.id))
            .set(in_template_data)
            .get_result::<HtyTemplateData<T>>(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn find_by_id(
        id_template_data: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<T>> {
        match hty_template_data::table
            .filter(hty_template_data::id.eq(id_template_data))
            .select(hty_template_data::all_columns)
            .first::<HtyTemplateData<T>>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn find_by_template_id_and_app_id(
        id_template: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyTemplateData<T>> {
        match hty_template_data::table
            .filter(
                hty_template_data::template_id
                    .eq(id_template)
                    .and(hty_template_data::app_id.eq(id_app)),
            )
            .select(hty_template_data::all_columns)
            .first::<HtyTemplateData<T>>(conn)
        {
            Ok(res) => Ok(res),
            Err(e) => Err({
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn delete_by_id(
        id_template_data: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<usize> {
        use crate::schema::hty_template_data::dsl::*;
        match diesel::delete(hty_template_data.find(id_template_data)).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_template_data::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }
}

#[derive(
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Insertable,
Debug,
Clone,
AsChangeset,
)]
#[diesel(table_name = hty_user_rels)]
#[diesel(primary_key(id))]
pub struct HtyUserRels {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub rel_type: String,
}

impl HtyUserRels {
    pub fn create(user_rel: &HtyUserRels, conn: &mut PgConnection) -> anyhow::Result<HtyUserRels> {
        use crate::schema::hty_user_rels::dsl::*;
        match insert_into(hty_user_rels)
            .values(user_rel)
            .get_result(conn)
        {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_all_with_params(
        in_rel_type: &Option<String>,
        in_from_user_id: &Option<String>,
        in_to_user_id: &Option<String>,
        conn: &mut PgConnection) -> anyhow::Result<Vec<HtyUserRels>> {
        debug!("find_all_with_params -> rel_type: {:?} / from_user_id: {:?} / to_user_id: {:?}", in_rel_type, in_from_user_id, in_to_user_id);

        use crate::schema::hty_user_rels::columns::*;
        let mut q = hty_user_rels::table.into_boxed();

        if let Some(rel_type_val) = in_rel_type.clone() {
            q = q.filter(rel_type.eq(rel_type_val));
        }

        if let Some(from_user_id_val) = in_from_user_id.clone() {
            q = q.filter(from_user_id.eq(from_user_id_val))
        }

        if let Some(to_user_id_val) = in_to_user_id.clone() {
            q = q.filter(to_user_id.eq(to_user_id_val))
        }

        match q.load::<HtyUserRels>(conn) {
            Ok(res) => Ok(res),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn find_by_all_col(id_user_from: &String, id_user_to: &String, type_rel: &String, conn: &mut PgConnection) -> anyhow::Result<HtyUserRels> {
        match hty_user_rels::table.filter(hty_user_rels::from_user_id.eq(id_user_from).and(hty_user_rels::to_user_id.eq(id_user_to)).and(hty_user_rels::rel_type.eq(type_rel)))
            .select(hty_user_rels::all_columns).first::<HtyUserRels>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                error!("HtyUserRels: find_by_all_col / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn delete_by_id(in_id: &String, conn: &mut PgConnection) -> bool {
        use crate::schema::hty_user_rels::dsl::*;
        match diesel::delete(hty_user_rels.find(in_id)).execute(conn) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[derive(
Associations,
AsChangeset,
Identifiable,
PartialEq,
Serialize,
Deserialize,
Queryable,
Debug,
Insertable,
Clone,
QueryableByName,
)]
#[diesel(belongs_to(HtyUserGroup, foreign_key = parent_id))]
#[diesel(belongs_to(HtyApp, foreign_key = app_id))]
#[diesel(table_name = hty_user_group)]
pub struct HtyUserGroup {
    pub id: String,
    pub users: Option<MultiVals<GroupUser>>, // it's hard to sync the user settings with hty_user, maybe reconsider whether to sync or not (currently not synced)
    pub group_type: String,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub app_id: String,
    pub group_name: String,
    pub is_delete: bool,
    pub group_desc: Option<String>,
    pub parent_id: Option<String>,
    pub owners: Option<MultiVals<GroupUser>>,
}

impl HtyUserGroup {
    pub fn find_by_id(
        id_user_group: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyUserGroup> {
        match hty_user_group::table
            .filter(hty_user_group::id.eq(id_user_group))
            .select(hty_user_group::all_columns)
            .first::<HtyUserGroup>(conn)
        {
            Ok(app) => Ok(app),
            Err(e) => Err({
                error!("find_by_id / err -> {:?}", e);
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            }),
        }
    }

    pub fn create(
        user_group: &HtyUserGroup,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyUserGroup> {
        use crate::schema::hty_user_group::dsl::*;
        match insert_into(hty_user_group)
            .values(user_group)
            .get_result(conn)
        {
            Ok(u) => Ok(u),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn update(
        in_user_group: &HtyUserGroup,
        conn: &mut PgConnection,
    ) -> anyhow::Result<HtyUserGroup> {
        let result = update(hty_user_group::table)
            .filter(hty_user_group::id.eq(in_user_group.clone().id))
            .set(in_user_group)
            .get_result(conn)
            .map_err(|e| {
                anyhow!(HtyErr {
                    code: HtyErrCode::DbErr,
                    reason: Some(e.to_string()),
                })
            });
        result
    }

    pub fn find_by_created_by_or_users(
        user: &String,
        some_is_delete: &Option<bool>,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyUserGroup>> {
        let q = if some_is_delete.is_some() {
            format!("select * from hty_user_group where is_delete = {} and (created_by = '{}' or jsonb_path_exists(users, '$.vals[*].user_id ? (@ == \"{}\")'))",
                    some_is_delete.clone().unwrap(),
                    user,
                    user).to_string()
        } else {
            format!("select * from hty_user_group where created_by = '{}' or jsonb_path_exists(users, '$.vals[*].user_id ? (@ == \"{}\")')", user, user).to_string()
        };

        let res = sql_query(q.clone()).load(conn)?;
        Ok(res)
    }

    pub fn find_by_app_id_or_users(
        user: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Option<Vec<HtyUserGroup>>> {
        let q = format!("select * from hty_user_group where app_id = '{}' and jsonb_path_exists(users, '$.vals[*].user_id ? (@ == \"{}\")')", id_app, user).to_string();
        let res = sql_query(q.clone()).load(conn).optional()?;
        Ok(res)
    }

    pub fn find_by_app_id_or_owners(
        id_owner: &String,
        id_app: &String,
        conn: &mut PgConnection,
    ) -> anyhow::Result<Vec<HtyUserGroup>> {
        let q = format!("select * from hty_user_group where app_id = '{}' and jsonb_path_exists(owners, '$.vals[*].user_id ? (@ == \"{}\")')", id_app, id_owner).to_string();
        let res = sql_query(q.clone()).load(conn)?;
        Ok(res)
    }

    pub fn find_all_active(conn: &mut PgConnection) -> anyhow::Result<Vec<HtyUserGroup>> {
        use crate::schema::hty_user_group::dsl::*;
        let res = hty_user_group
            .filter(is_delete.eq(false))
            .load::<HtyUserGroup>(conn)?;
        Ok(res)
    }

    pub fn delete_all(conn: &mut PgConnection) -> anyhow::Result<usize> {
        match diesel::delete(hty_user_group::table).execute(conn) {
            Ok(num) => Ok(num),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(e.to_string()),
            })),
        }
    }

    pub fn to_req(&self) -> ReqHtyUserGroup {
        ReqHtyUserGroup {
            id: Some(self.id.clone()),
            users: self.users.clone(),
            group_type: Some(self.group_type.clone()),
            created_at: self.created_at.clone(),
            created_by: self.created_by.clone(),
            app_id: Some(self.app_id.clone()),
            group_name: Some(self.group_name.clone()),
            is_delete: Some(self.is_delete.clone()),
            group_desc: self.group_desc.clone(),
            parent_id: self.parent_id.clone(),
            owners: self.owners.clone(),
        }
    }
}

//
// // 因为泛型的原因，这里不能用这个macro了
// // impl_jsonb_boilerplate!(MultiVals);
// impl<T: Debug + Serialize + DeserializeOwned> diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg>
// for MultiVals<T>
// {
//     fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
//         let value = <serde_json::Value as diesel::deserialize::FromSql<
//             diesel::sql_types::Jsonb,
//             diesel::pg::Pg,
//         >>::from_sql(bytes)?;
//         Ok(serde_json::from_value(value)?)
//     }
// }
//
// impl<T: Debug + Serialize + DeserializeOwned> diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for MultiVals<T> {
//     fn to_sql<W: std::io::Write>(
//         &self,
//         out: &mut diesel::serialize::Output<W, diesel::pg::Pg>,
//     ) -> diesel::serialize::Result {
//         let value = serde_json::to_value(self)?;
//         <serde_json::Value as diesel::serialize::ToSql<
//             diesel::sql_types::Jsonb,
//             diesel::pg::Pg,
//         >>::to_sql(&value, out)
//     }
// }

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct GroupUser {
    pub user_id: Option<String>,
    pub real_name: Option<String>,
    pub user_type: Option<String>, // 用户在这个小组里的角色类型：STUDENT / TEACHER / ...
    pub settings: Option<MultiVals<UserSetting>>, // it's hard to sync the user settings with hty_user, maybe reconsider whether to sync or not (currently not synced)
    pub role_id: Option<String>,
}

impl_jsonb_boilerplate!(GroupUser);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReqHtyUserRels {
    pub id: Option<String>,
    pub from_user_id: Option<String>,
    pub from_user_realname: Option<String>,
    pub to_user_id: Option<String>,
    pub to_user_realname: Option<String>,
    pub rel_type: Option<String>,
}


// deprecated, use `CommonTask` instead.
