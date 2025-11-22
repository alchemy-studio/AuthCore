use std::ops::Deref;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use tracing::debug;
use htycommons::common::{HtyErr, HtyErrCode, HtyResponse};
use htycommons::web::{get_uc_url, HtyHostHeader, HtySudoerTokenHeader};
use htyuc_models::models::{HtyApp, ReqAppFromTo, ReqHtyApp, ReqHtyTemplate, ReqHtyTongzhi, ReqHtyUserWithInfos};

pub async fn find_to_apps_by_domain(
    root: &HtySudoerTokenHeader,
    host: &HtyHostHeader,
) -> anyhow::Result<Vec<ReqAppFromTo>> {
    debug!("find_to_apps_by_domain -> root: {:?} / host: {:?}", root, host);

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/find_to_apps_by_domain", get_uc_url(), ))
        .header("HtySudoerToken", root.deref().to_string())
        .header("HtyHost", host.deref().to_string())
        .send()
        .await?;

    debug!("find_to_apps_by_domain -> resp: {:?}", resp);

    let decode_resp = resp.json::<HtyResponse<Vec<ReqAppFromTo>>>().await?;

    debug!("find_to_apps_by_domain -> decode_resp: {:?}", decode_resp);

    let to_apps = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some("to_apps not found".to_string()),
    })?;

    debug!("find_to_apps_by_domain -> to_apps: {:?}", to_apps);
    Ok(to_apps)
}

pub async fn find_hty_user_with_info_by_id(
    id_hty: &String,
    root: &HtySudoerTokenHeader,
) -> anyhow::Result<ReqHtyUserWithInfos> {
    debug!("-> begin");

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/find_user_with_info_by_id/{}",
            get_uc_url(),
            id_hty
        ))
        .header("HtySudoerToken", root.deref().to_string())
        .send()
        .await?;
    let decode_resp = resp.json::<HtyResponse<ReqHtyUserWithInfos>>().await?;

    debug!("find_hty_user_with_info_by_id / resp -> {:?}", &decode_resp);

    let res = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some(format!("hty_user not found: hty_id -> {:?}", id_hty)),
    })?;
    Ok(res)
}

pub async fn find_app_by_domain(
    root: &HtySudoerTokenHeader,
    host: &HtyHostHeader,
) -> anyhow::Result<HtyApp> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/find_app_by_domain", get_uc_url(), ))
        .header("HtySudoerToken", root.deref().to_string())
        .header("HtyHost", host.deref().to_string())
        .send()
        .await?;
    let decode_resp = resp.json::<HtyResponse<ReqHtyApp>>().await?;
    let req_app = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some("app not found".to_string()),
    })?;

    let app_id = req_app.app_id
        .ok_or_else(|| anyhow::anyhow!("app_id is required"))?;
    let app_status = req_app.app_status
        .ok_or_else(|| anyhow::anyhow!("app_status is required"))?;
    Ok(HtyApp {
        app_id,
        wx_secret: req_app.wx_secret,
        domain: req_app.domain,
        app_desc: req_app.app_desc,
        app_status,
        pubkey: req_app.pubkey,
        privkey: req_app.privkey,
        wx_id: req_app.wx_id,
        is_wx_app: req_app.is_wx_app,
    })
}

pub async fn find_app_by_id(id_app: &String, root: &HtySudoerTokenHeader) -> anyhow::Result<HtyApp> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/find_app_by_id/{}",
            get_uc_url(),
            id_app
        ))
        .header("HtySudoerToken", root.deref().to_string())
        .send()
        .await?;
    let decode_resp = resp.json::<HtyResponse<ReqHtyApp>>().await?;
    let req_app = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some("to_app not found".to_string()),
    })?;

    let app_id = req_app.app_id
        .ok_or_else(|| anyhow::anyhow!("app_id is required"))?;
    let app_status = req_app.app_status
        .ok_or_else(|| anyhow::anyhow!("app_status is required"))?;
    Ok(HtyApp {
        app_id,
        wx_secret: req_app.wx_secret,
        domain: req_app.domain,
        app_desc: req_app.app_desc,
        app_status,
        pubkey: req_app.pubkey,
        privkey: req_app.privkey,
        wx_id: req_app.wx_id,
        is_wx_app: req_app.is_wx_app,
    })
}

pub async fn find_user_openid_by_hty_id_and_app(
    hty_id: &String,
    to_app: &HtyApp,
    root: &HtySudoerTokenHeader,
) -> anyhow::Result<String> {
    let hty_user = find_hty_user_with_info_by_id(hty_id, root).await?;
    let req_user_infos = hty_user.infos.as_ref();
    debug!(
        "find_user_openid_by_hty_id_and_app() -> find user {:?} with info {:?}",
        hty_user, req_user_infos
    );

    let user_infos = req_user_infos
        .ok_or_else(|| anyhow::anyhow!("user_infos is required"))?;
    let to_user_openid = user_infos
        .iter()
        .find(|user_info| user_info.app_id.as_ref().map(|id| id == &to_app.app_id).unwrap_or(false))
        .ok_or(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some("not found a user_app_info to_app!".into()),
        })?
        .openid
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("openid is required"))?;
    Ok(to_user_openid.to_string())
}

pub async fn create_tongzhi(
    req_tongzhi: &ReqHtyTongzhi,
    root: &HtySudoerTokenHeader,
) -> anyhow::Result<String> {
    debug!("create_tongzhi() -> req tongzhi: {:?}", req_tongzhi);
    let body = serde_json::to_string::<ReqHtyTongzhi>(req_tongzhi)?;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/create_tongzhi", get_uc_url(), ))
        .body(body)
        .header("HtySudoerToken", root.deref().to_string())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .send()
        .await?;

    let decode_resp = resp.json::<HtyResponse<String>>().await?;
    debug!("remote_calls / create_tongzhi: resp -> {:?}", decode_resp);
    let res = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some(String::from("Fail to create tongzhi")),
    })?;
    Ok(res)
}

//----------------------------------------------------------------------------------------------------

pub async fn get_template_with_data_by_key_and_app_id(
    key_template: &String,
    id_app: &String,
    root: &HtySudoerTokenHeader,
) -> anyhow::Result<ReqHtyTemplate<String>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/find_hty_template_with_data_by_key_and_app_id?app_id={}&template_key={}",
            get_uc_url(),
            id_app,
            key_template,
        ))
        .header("HtySudoerToken", root.deref().to_string())
        .send()
        .await?;

    let decode_resp = resp.json::<HtyResponse<ReqHtyTemplate<String>>>().await?;

    let ret = decode_resp.d.ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some(format!(
            "template not found -> id_app: {:?} / key_template: {:?}",
            id_app, key_template
        )),
    })?;
    Ok(ret)
}