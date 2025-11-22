use std::fmt::Debug;
// use std::collections::HashMap;
use std::future::Future;
// use std::pin::Pin;
// use std::pin::Pin;
use anyhow::anyhow;
use crate::models::{HtyUser, UserAppInfo, HtyApp, ReqHtyTemplate, HtyTemplateData};
use diesel::PgConnection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::debug;
use htycommons::uuid;
use htycommons::redis_util::{all_openids_prefix, get_value_from_redis, is_key_exist_in_redis, openid_info_prefix, save_kv_to_redis, save_kv_to_redis_with_exp_secs, WX_ACCESS_TOKEN_PREFIX, WX_JSAPI_TICKET_PREFIX};
use htycommons::web::{HtyToken, skip_wx_push};
use htycommons::common::{current_local_datetime, HtyErr, HtyErrCode};
use htycommons::wx::{ReqWxAccessToken, ReqWxAccessToken1, ReqWxAllFollowers, ReqWxFollowerInfo, ReqWxPushMessage, ReqWxPushResponse, ReqWxTicket, WxId};


// frontend passed in unionid, and use it directly.
pub fn identify2(id: &WxId, app_id: &String, conn: &mut PgConnection) -> anyhow::Result<HtyToken> {
    let hty_id;
    match HtyUser::find_by_union_id(&id.union_id, conn) {
        Ok(user) => {
            hty_id = user.hty_id.clone();
            // 返回jwt token
            Ok(HtyToken {
                token_id: uuid(),
                hty_id: Some(hty_id),
                app_id: None,
                ts: current_local_datetime(),
                roles: user
                    .info(app_id, conn)?
                    .req_roles_by_id(conn)?,
                tags: None,
            })
        }
        Err(_) => {
            let new_user = HtyUser {
                hty_id: "".to_string(),
                union_id: Some(id.union_id.clone()),
                enabled: true,
                created_at: None,
                real_name: None,
                sex: None,
                mobile: None,
                settings: None,
            };

            let new_info = UserAppInfo {
                id: "".into(),
                hty_id: "".into(),
                app_id: Some(app_id.clone()),
                openid: Some(id.openid.clone()),
                is_registered: false,
                username: Some(uuid()),
                password: None,
                meta: None,
                created_at: None,
                teacher_info: None,
                student_info: None,
                reject_reason: None,
                needs_refresh: Some(false),
                avatar_url: None,
            };

            hty_id = HtyUser::create_with_info_with_tx(&new_user, &Some(new_info), conn)
                .ok_or_else(|| anyhow::anyhow!("Failed to create user with info"))?
                .clone();

            Ok(HtyToken {
                token_id: uuid(),
                hty_id: Some(hty_id),
                app_id: None,
                ts: current_local_datetime(),
                roles: None,
                tags: None,
            })
        }
    }
}

// todo : 需要处理缓存里面token已经在微信那边过期的问题（微信默认过期时间为2个小时）
pub async fn get_or_save_wx_access_token(app: &HtyApp, force_refresh: bool) -> anyhow::Result<String> {
    let mut token_id = WX_ACCESS_TOKEN_PREFIX.to_string();
    token_id.push_str(app.clone().app_id.as_str());

    if !force_refresh
    {
        let is_token_available = is_key_exist_in_redis(&token_id)?;
        if is_token_available {
            return get_value_from_redis(&token_id);
        }
    }

    // force_refresh = true or
    // token not in cache, refresh the token by accessing WX API.
    let client = reqwest::Client::new();
    let id_wx = app.wx_id.clone()
        .ok_or_else(|| anyhow::anyhow!("wx_id is required"))?;
    let secret = app.wx_secret.clone()
        .ok_or_else(|| anyhow::anyhow!("wx_secret is required"))?;

    let url = "https://api.weixin.qq.com/cgi-bin/token";
    debug!("app -> {:?}, wx_id = {:?}, secret = {:?}", app.app_desc.clone(), id_wx.clone(), secret.clone());
    let resp = client
        .get(url)
        .query(&[("grant_type", "client_credential"), ("appid", id_wx.clone().as_str()), ("secret", secret.as_str())])
        .send().await?
        .text().await?;

    debug!("get_access_token -> {}", resp);

    let access_token = serde_json::from_str::<ReqWxAccessToken>(
        resp.as_str(),
    )?;

    debug!("get_access_token -> access_token -> {:?}", access_token);

    if access_token.errcode.is_some() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(format!("Get Wx access token met error. Error code: {:?}, Error msg: {:?} ", access_token.errcode, access_token.errmsg).to_string())
        }));
    }
    let access_token_str = access_token.access_token
        .ok_or_else(|| anyhow::anyhow!("access_token is missing in response"))?;
    let exp = access_token.expires_in
        .ok_or_else(|| anyhow::anyhow!("expires_in is missing in response"))? / 2;
    save_kv_to_redis_with_exp_secs(&token_id, &access_token_str, exp)?;
    Ok(access_token_str)
}

async fn fn_get_jsapi_ticket(app: Option<HtyApp>, _: Option<ReqWxPushMessage<()>>, _: Option<String>, token: String) -> anyhow::Result<(Option<String>, Option<i32>, Option<String>)> {
    let app_ref = app.as_ref()
        .ok_or_else(|| anyhow::anyhow!("app is required"))?;
    let mut token_id = WX_JSAPI_TICKET_PREFIX.to_string();
    token_id.push_str(app_ref.app_id.as_str());

    let is_ticket_available = is_key_exist_in_redis(&token_id)?;

    if is_ticket_available {
        return Ok((Some(get_value_from_redis(&token_id)?), None, None));
    }

    let client = reqwest::Client::new();
    // let token = get_or_save_wx_access_token(&app.unwrap(), false).await?;

    let url = "https://api.weixin.qq.com/cgi-bin/ticket/getticket";
    let resp = client
        .get(url)
        .query(&[("access_token", token.as_str()), ("type", "jsapi")])
        .send().await?.text().await?;

    debug!("fn_get_jsapi_ticket -> receive resp -> {:?}", resp);

    let jsapi_ticket = serde_json::from_str::<ReqWxTicket>(
        resp.as_str()
    )?;

    debug!("fn_get_jsapi_ticket -> wx_ticket -> {:?}", jsapi_ticket);

    if let Some(errcode) = jsapi_ticket.errcode.clone() {
        if errcode != 0 {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::WebErr,
                reason: Some(format!("Get Wx jsapi_ticket met error. Error code: {:?}, Error msg: {:?} ", jsapi_ticket.errcode, jsapi_ticket.errmsg).to_string())
            }));
        }
    }
    let ticket = jsapi_ticket.ticket
        .ok_or_else(|| anyhow::anyhow!("ticket is missing in response"))?;
    let exp = jsapi_ticket.expires_in
        .ok_or_else(|| anyhow::anyhow!("expires_in is missing in response"))?;

    save_kv_to_redis_with_exp_secs(&token_id, &ticket, exp)?;

    Ok((Some(ticket), jsapi_ticket.errcode, jsapi_ticket.errmsg))
}

pub async fn get_jsapi_ticket(app: &HtyApp) -> anyhow::Result<String> {
    get_access_code_and_call_wx_func(fn_get_jsapi_ticket, Some(app.clone()), None, None).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to get jsapi_ticket"))
}

pub async fn get_cached_wx_all_follower_openids(app: &HtyApp) -> anyhow::Result<Vec<String>> {
    Ok(serde_json::from_str::<Vec<String>>(get_value_from_redis(&all_openids_prefix(&app.app_id))?.as_str())?)
}

pub async fn refresh_cache_and_get_wx_all_follower_openids(app: &HtyApp) -> anyhow::Result<Vec<String>> {
    // let token = get_or_save_wx_access_token(app, false).await?;

    let none: Option<ReqWxPushMessage<()>> = None;
    get_access_code_and_call_wx_func(fn_refresh_cache_and_get_wx_all_follower_openids, Some(app.clone()), none, None).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to get follower openids"))
}

pub async fn fn_refresh_cache_and_get_wx_all_follower_openids<T: Send + Clone + Serialize + Debug>(app: Option<HtyApp>,
                                                                                                   _: Option<ReqWxPushMessage<T>>,
                                                                                                   _: Option<String>,
                                                                                                   token: String) -> anyhow::Result<(Option<Vec<String>>, Option<i32>, Option<String>)> {
    let url = "https://api.weixin.qq.com/cgi-bin/user/get";

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&[("access_token", token.as_str()), ("next_openid", "")])
        .send().await?;

    if resp.status().as_str() == "200" {
        let resp_body = resp.text().await?;
        let req_followers = serde_json::from_str::<ReqWxAllFollowers>(
            resp_body.as_str(),
        )?;
        debug!("get_wx_all_followers -> get resp from weixin {:?}", req_followers);
        let openids = req_followers.data.openid
            .ok_or_else(|| anyhow::anyhow!("openid is missing in response"))?;
        let json_openids = serde_json::to_string(&openids)?;
        let app_ref = app.as_ref()
            .ok_or_else(|| anyhow::anyhow!("app is required"))?;
        let _ = save_kv_to_redis(&all_openids_prefix(&app_ref.app_id), &json_openids)?;
        Ok((Some(openids), req_followers.errcode.clone(), req_followers.errmsg.clone()))
    } else {
        let resp_body = resp.text().await?;
        Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(format!("Get all followers met error. Error msg: {:?} ", resp_body).to_string())
        }))
    }
}


pub async fn get_cached_wx_follower_info(openid: &String, app: &HtyApp) -> anyhow::Result<ReqWxFollowerInfo> {
    let info = serde_json::from_str::<ReqWxFollowerInfo>(get_value_from_redis(&openid_info_prefix(openid, &app.app_id))?.as_str())?;
    debug!("get_cached_wx_follower_info -> OPENID {:?} / APP {:?} / INFO {:?}", openid, app, info);
    Ok(info)
}

pub async fn fn_refresh_and_get_wx_follower_info<T: Send + Clone + Serialize + Debug>(
    app: Option<HtyApp>,
    _: Option<ReqWxPushMessage<T>>,
    openid: Option<String>,
    token: String) -> anyhow::Result<(Option<ReqWxFollowerInfo>, Option<i32>, Option<String>)> {
// let token = get_or_save_wx_access_token(app, false).await?;

    let url = "https://api.weixin.qq.com/cgi-bin/user/info";

    let openid_str = openid
        .ok_or_else(|| anyhow::anyhow!("openid is required"))?;
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .query(&[("access_token", token.as_str()), ("openid", openid_str.as_str())])
        .send()
        .await?;

    if resp.status().as_str() == "200" {
        let resp_body = resp.text().await?;
        let json_follower_info = resp_body.clone();
        let req_follower_info = serde_json::from_str::<ReqWxFollowerInfo>(
            json_follower_info.as_str()
        )?;

        debug!("fn_refresh_and_get_wx_follower_info -> get follower info {:?}", req_follower_info);

        let app_ref = app.as_ref()
            .ok_or_else(|| anyhow::anyhow!("app is required"))?;
        let _ = save_kv_to_redis(&openid_info_prefix(&req_follower_info.clone().openid, &app_ref.app_id),
                                 &json_follower_info.to_string())?;

        Ok((Some(req_follower_info.clone()), req_follower_info.errcode.clone(), req_follower_info.errmsg.clone()))
    } else {
        let resp_body = resp.text().await?;
        Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(format!("Get follower info met error. Error msg: {:?} ", resp_body).to_string())
        }))
    }
}

pub async fn refresh_and_get_wx_follower_info(openid: &String, app: &HtyApp) -> anyhow::Result<ReqWxFollowerInfo> {
    let none: Option<ReqWxPushMessage<()>> = None;
    get_access_code_and_call_wx_func(fn_refresh_and_get_wx_follower_info, Some(app.clone()), none, Some(openid.clone())).await?
        .ok_or_else(|| anyhow::anyhow!("Failed to get follower info"))
}


pub async fn get_cached_or_refresh_wx_follower_info(openid: &String, app: &HtyApp) -> anyhow::Result<ReqWxFollowerInfo> {
    // 如果有缓存数据，直接使用；如果没有，刷新缓存并返回。
    debug!("get_cached_or_refresh_wx_follower_info start");
    match get_cached_wx_follower_info(openid, app).await {
        Ok(info) =>
            {
                debug!("get_cached_or_refresh_wx_follower_info FOUND CACHE INFO -> {:?}", info);
                Ok(info)
            }
        Err(_) => {
            debug!("get_cached_or_refresh_wx_follower_info NOT FOUND CACHE INFO, fetch from WX");
            refresh_and_get_wx_follower_info(openid, app).await
        }
    }
}

pub async fn find_wx_openid_by_unionid_and_hty_app(union_id: &String, app: &HtyApp) -> anyhow::Result<String> {
    let follower_openids = get_cached_wx_all_follower_openids(&app).await?;
    let mut follower_infos = vec![];

    for follower_openid in follower_openids {
        follower_infos.push(get_cached_or_refresh_wx_follower_info(&follower_openid, &app).await?);
    }

    let res_follower_info = follower_infos
        .into_iter()
        .find(|follower_info| follower_info.unionid == union_id.clone())
        .ok_or(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some(format!("这个用户没有对应TO_APP的OPENID！此用户UNIONID: {:?}", union_id)),
        })?;

    Ok(res_follower_info.openid)
}

//
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8435f34f94963c32a9c990be950cd27d
// https://stackoverflow.com/questions/66769143/rust-passing-async-function-pointers
// 不能为递归函数，只能用循环，否则`wx_func`的type搞不定(dyn + Box各种问题)
pub async fn get_access_code_and_call_wx_func<F, R, U, V>(wx_func: F,
                                                          to_app: Option<HtyApp>,
                                                          wx_push_message: Option<ReqWxPushMessage<U>>,
                                                          openid: Option<String>) -> anyhow::Result<Option<V>>
    where
        F: Fn(Option<HtyApp>, Option<ReqWxPushMessage<U>>, Option<String>, String) -> R, // String = `wx_token` from `get_or_save_wx_access_token` / Option<String> = openid
        R: Future<Output=anyhow::Result<(Option<V>, Option<i32>, Option<String>)>> + Send,
        U: Clone + Serialize + Debug,
        V: Clone + Serialize + Debug
{
    debug!("get_access_code_and_call_wx_func -> to_app: {:?} / wx_push_message: {:?}", to_app, wx_push_message);

    let mut ok = false;
    let mut retry = 0;
    let mut some_wx_call_resp = None;

    let to_app_ref = to_app.as_ref()
        .ok_or_else(|| anyhow::anyhow!("to_app is required"))?;
    
    while retry < 3 && !ok {
        let wx_token = if retry == 0 {
            get_or_save_wx_access_token(to_app_ref, false).await?
        } else {
            get_or_save_wx_access_token(to_app_ref, true).await?
        };

        let (wx_call_resp, wx_call_err_code, wx_call_err_msg) = wx_func(to_app.clone(), wx_push_message.clone(), openid.clone(), wx_token).await?;
        debug!("get_access_code_and_call_wx_func -> OK / wx_call_resp: {:?} / wx_call_err_code: {:?} / wx_call_err_msg: {:?}", wx_call_resp, wx_call_err_code, wx_call_err_msg);


        if wx_call_err_code.is_none() || (wx_call_err_code.is_some() && wx_call_err_code.as_ref().unwrap() == &0) {
            ok = true;
            some_wx_call_resp = wx_call_resp;
        } else {
            debug!("get_access_code_and_call_wx_func -> retry {:?} / err -> {:?}", retry, wx_call_err_msg);
            retry = retry + 1;
        };
    }

    return if !ok {
        Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(format!("get_access_code_and_call_wx_func_func: EXCEEDS MAX RETRIES")
                .to_string())}))
    } else {
        if some_wx_call_resp.is_none() {
            Ok(None)
        } else {
            Ok(some_wx_call_resp)
        }
    };
}

// pub type WxCallFunc<T: Clone + Serialize> = dyn Fn(Option<HtyApp>, Option<ReqWxPushMessage<T>>, String) -> Future<Output=anyhow::Result<ReqWxPushResponse>> + Send;

// pub async fn get_access_code_and_call_wx_func<T: Clone + Serialize>(wx_func: Pin<Box<WxCallFunc<T>>>,
//                                                                        to_app: &Option<HtyApp>,
//                                                                        wx_push_message: &Option<ReqWxPushMessage<T>>)
//     -> anyhow::Result<ReqWxPushResponse> {
//     raw_get_access_code_and_call_wx_func(wx_func, to_app, wx_push_message, 0).await
// }
//
// async fn raw_get_access_code_and_call_wx_func<T: Clone>(wx_func: Pin<Box<WxCallFunc<T>>>,
//                                                            to_app: &Option<HtyApp>,
//                                                            wx_push_message: &Option<ReqWxPushMessage<T>>,
//                                                            retry: u32) -> anyhow::Result<ReqWxPushResponse> {
//     if retry > 3 {
//         return Err(anyhow!(HtyErr {
//             code: HtyErrCode::WebErr,
//             reason: Some(format!("raw_get_access_code_and_call_wx_func: raw_get_access_code_and_call_wx_func EXCEEDS MAX RETRIES")
//                 .to_string())}));
//     }
//
//     let wx_token = if retry == 0 {
//         get_or_save_wx_access_token(&to_app.clone().unwrap(), false).await?
//     } else {
//         get_or_save_wx_access_token(&to_app.clone().unwrap(), true).await?
//     };
//
//     let wx_push_resp = wx_func(to_app.clone(), wx_push_message.clone(), wx_token).await?;
//
//     return if wx_push_resp.errcode.is_some() {
//         raw_get_access_code_and_call_wx_func(wx_func, to_app, wx_push_message, retry + 1).await
//     } else {
//         Ok(wx_push_resp)
//     };
// }

pub async fn fn_push_wx_message<T: Send + Clone + Serialize + Debug>(
    _to_app: Option<HtyApp>,
    wx_push_message: Option<ReqWxPushMessage<T>>,
    _openid: Option<String>,
    token: String,
) -> anyhow::Result<(Option<ReqWxPushResponse>, Option<i32>, Option<String>)> {
    // todo: check NULL of wx_push_message
    debug!("fn_push_wx_message -> to_app: {:?} / wx_push_message: {:?} / token: {:?}", _to_app, wx_push_message, token);
    let wx_url = "https://api.weixin.qq.com/cgi-bin/message/template/send";
    let wx_push_message_ref = wx_push_message
        .ok_or_else(|| anyhow::anyhow!("wx_push_message is required"))?;
    let post_body = serde_json::to_string::<ReqWxPushMessage<T>>(wx_push_message_ref)?;

    debug!("fn_push_wx_message -> post wx body {:?} ", post_body);

    let client = reqwest::Client::new();

    let resp = client
        .post(wx_url)
        .query(&[("access_token", token.as_str())])
        .body(post_body)
        .send().await?;

    let resp_body = resp.text().await?;
    debug!("fn_push_wx_message() -> wx response body {:?} ", resp_body);

    let wx_push_resp = serde_json::from_str::<ReqWxPushResponse>(
        resp_body.as_str(),
    )?;

    Ok((Some(wx_push_resp.clone()), wx_push_resp.errcode.clone(), wx_push_resp.errmsg.clone()))
}

pub async fn push_wx_message<T: Serialize + Clone + Send + Debug>(to_app: &HtyApp, wx_push_message: &ReqWxPushMessage<T>) -> anyhow::Result<()> {
    debug!("push_wx_message START -> to_app: {:?} / wx_push_message: {:?}", to_app, wx_push_message);

    if skip_wx_push().unwrap_or(false) {
        debug!("push_wx_message() ::BYPASSED::");
        Ok(())
    } else {
        if wx_push_message.touser.is_none() || wx_push_message.touser.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            debug!("push_wx_message() ::TO_USER_OPENID IS NULL:: 无法推送消息");
            Ok(())
        } else {
            let wx_push_resp = get_access_code_and_call_wx_func(fn_push_wx_message,
                                                                Some(to_app.clone()),
                                                                Some(wx_push_message.clone()),
                                                                None).await?
                .ok_or_else(|| anyhow::anyhow!("Failed to push wx message"))?;
            debug!("push_wx_message -> wx_push_resp -> {:?}", wx_push_resp);
            let resp_code = wx_push_resp.errcode
                .ok_or_else(|| anyhow::anyhow!("errcode is missing in response"))?;
            if resp_code == 0 {
                Ok(())
            } else {
                Err(anyhow!(HtyErr {code: HtyErrCode::WebErr,reason: Some(format!("push_wx_message met error. Error msg: {:?} ", wx_push_resp.errmsg).to_string())}))
            }
        }
    }
}
#[macro_export]
macro_rules! remote_send_tongzhi_and_push_wx_message {
    ($req_hty_tongzhi: ident, $sudoer_copy: ident, $to_app: ident, $push_message: ident) => {
        use ::htyuc_models::wx::push_wx_message2;
        ::tokio::spawn(async move {
            // huiwingn内部通知（小程序内部）
            let r_created_tongzhi_id = create_tongzhi(&$req_hty_tongzhi, &$sudoer_copy).await;
            // 推送公众号小程序通知.
            if let Ok(tongzhi_id) = r_created_tongzhi_id {
                let _ = ::htyuc_models::wx::push_wx_message2(&$to_app, &$push_message, Some(tongzhi_id)).await;
            }
        });
    }
}

#[macro_export]
macro_rules! send_tongzhi {
    ($hty_tongzhi: ident, $in_to_app: ident, $push_message: ident, $db_pool: ident) => {
        use ::htyuc_models::wx::push_wx_message2;
        use ::htyuc_models::models::HtyTongzhi;

        let to_app = $in_to_app.clone();

        let created_tongzhi = ::htyuc_models::models::HtyTongzhi::create(&$hty_tongzhi, extract_conn(fetch_db_conn(&$db_pool)?).deref_mut())?;

        ::tokio::spawn(async move {
            let _ = ::htyuc_models::wx::push_wx_message2(&to_app, &$push_message, Some(created_tongzhi.tongzhi_id)).await;
        });

        // // use ::htyuc_models::wx::push_wx_message2;
        // ::tokio::spawn(async move {
        //     let some_created_tongzhi_id = create_tongzhi(&$hty_tongzhi, &$sudoer_copy).await;
        //     let _ = ::htyuc_models::wx::push_wx_message2(&$to_app, &$push_message, Some(some_created_tongzhi_id.unwrap())).await;
        // });
    }
}

pub async fn push_wx_message2<T: Serialize + Clone + Send + Debug>(to_app: &HtyApp, in_wx_push_message: &ReqWxPushMessage<T>, some_created_hty_tongzhi_id: Option<String>) -> anyhow::Result<()> {
    debug!("push_wx_message START -> to_app: {:?} / in_wx_push_message: {:?}", to_app, in_wx_push_message);

    let mut wx_push_message = in_wx_push_message.clone();

    if let (Some(tongzhi_id), Some(miniprogram)) = (some_created_hty_tongzhi_id.as_ref(), in_wx_push_message.miniprogram.as_ref()) {
        let mut c_miniprogram = miniprogram.clone();
        c_miniprogram.pagepath = format!("pages/index/index?hty_tongzhi_id={}", tongzhi_id);
        // c_miniprogram.path = format!("sample?hty_tongzhi_id={}", tongzhi_id);
        wx_push_message.miniprogram = Some(c_miniprogram);
    }

    debug!("push_wx_message2 -> {:?}", wx_push_message);

    if skip_wx_push().unwrap_or(false) {
        debug!("push_wx_message2() ::BYPASSED::");
        Ok(())
    } else {
        if wx_push_message.touser.is_none() || wx_push_message.touser.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            debug!("push_wx_message2() ::TO_USER_OPENID IS NULL:: 无法推送消息");
            Ok(())
        } else {
            let wx_push_resp = get_access_code_and_call_wx_func(fn_push_wx_message,
                                                                Some(to_app.clone()),
                                                                Some(wx_push_message.clone()),
                                                                None).await?
                .ok_or_else(|| anyhow::anyhow!("Failed to push wx message"))?;
            debug!("push_wx_message2 -> wx_push_resp: {:?}", wx_push_resp);
            let resp_code = wx_push_resp.errcode
                .ok_or_else(|| anyhow::anyhow!("errcode is missing in response"))?;
            if resp_code == 0 {
                Ok(())
            } else {
                Err(anyhow!(HtyErr {code: HtyErrCode::WebErr,reason: Some(format!("push_wx_message2 met error. Error msg: {:?} ", wx_push_resp.errmsg).to_string())}))
            }
        }
    }
}

pub async fn get_union_id_by_auth_code(wx_id: String, secret: String, code: String) -> anyhow::Result<String> {
    let token_url = "https://api.weixin.qq.com/sns/oauth2/access_token";

    let client = reqwest::Client::new();
    let resp = client
        .get(token_url)
        .query(&[("grant_type", "authorization_code"), ("appid", wx_id.as_str()), ("secret", secret.as_str()), ("code", code.as_str())])
        .send().await?
        .text().await?;

    debug!("get_union_id_by_auth_code -> resp -> {}", resp);

    let access_token = serde_json::from_str::<ReqWxAccessToken1>(
        resp.as_str(),
    )?;

    debug!("get_union_id_by_auth_code -> access_token -> {:?}", access_token);

    if access_token.errcode.is_some() {
        return Err(anyhow!(HtyErr {
            code: HtyErrCode::WebErr,
            reason: Some(format!("Get Wx access token met error. Error code: {:?}, Error msg: {:?} ", access_token.errcode, access_token.errmsg).to_string())
        }));
    }

    let union_id_str = access_token.unionid
        .ok_or_else(|| anyhow::anyhow!("unionid is missing in response"))?;

    debug!("get_union_id_by_auth_code -> union_id_str -> {:?}", union_id_str);

    Ok(union_id_str)
}

pub fn extract_template_data_and_wx_message_data_from_template<
    T: Debug + Serialize + DeserializeOwned + Clone,
>(
    req_in_template: &ReqHtyTemplate<String>,
) -> anyhow::Result<(HtyTemplateData<String>, T)> {
    debug!(
        "extract_template_data_and_wx_message_data_from_template -> {:?}",
        req_in_template
    );

    let datas = req_in_template
        .datas
        .clone()
        .ok_or_else(|| anyhow::anyhow!("datas is required"))?;
    let first_data = datas
        .get(0) // just assume we have only one template data.
        .ok_or_else(|| anyhow::anyhow!("at least one template data is required"))?
        .clone();
    let in_template_data = first_data.to_db_struct()?;
    let raw_wx_message_text = in_template_data
        .template_text
        .clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;
    // let wrapped_data = SingleVal {
    //     val: Some(raw_wx_message_data)
    // };

    // let extracted_data = serde_json::from_str::<SingleVal<T>>(serde_json::to_string(&wrapped_data)?.as_str())?.val.unwrap();
    let extracted_template_text = serde_json::from_str::<T>(raw_wx_message_text.as_str())?;
    Ok((in_template_data, extracted_template_text))
}