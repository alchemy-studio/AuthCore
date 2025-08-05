use crypto::digest::Digest;
use crypto::md5::Md5;
use data_encoding::BASE64;
use ring::hmac;
use std::env;
use crate::web::get_ngx_url;
use log::{debug};

#[derive(Deserialize, Serialize)]
pub struct UpyunParams {
    pub uri: String,
    pub method: String,
    pub date: String,
    pub expiration: i32,
}

#[derive(Debug, Serialize)]
pub struct UpYunAuth {
    pub auth: String,
    pub sign: String,
    pub policy: String,
}

#[derive(Serialize)]
pub struct Policy {
    pub bucket: String,
    pub expiration: i32,
    #[serde(rename = "save-key")]
    pub save_key: String,
}

/// 又拍云token
pub fn generate_upyun_token(data: &String, operator: &String, password: &String) -> String {
    let mut hasher = Md5::new();
    hasher.input(password.clone().as_bytes());
    let md5_pwd = hasher.result_str();
    sign(operator, &md5_pwd, data)
}

fn sign(operator: &String, md5_pwd: &String, data: &String) -> String {
    let signed_key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, md5_pwd.as_bytes());
    let auth = hmac::sign(&signed_key, data.as_bytes());
    let token = BASE64.encode(auth.as_ref());
    format!("UPYUN {}:{}", operator, token)
}

pub fn get_upyun_operator() -> String {
    env::var("UPYUN_OPERATOR").expect("UPYUN_OPERATOR not set⚠️")
}

pub fn get_upyun_password() -> String {
    env::var("UPYUN_PASSWORD").expect("UPYUN_PASSWORD not set⚠️")
}
// pub fn generate_upyun_token(upyun_params: &UpyunParams) -> UpYunAuth {
//     let mut hasher1 = Md5::new();
//     let policy = Policy {
//         expiration: upyun_params.expiration,
//         save_key: upyun_params.uri.to_string(),
//         bucket: format!("{}", "huiwing"),
//     };
//     hasher1.input(env::var("OPERATOR_PWD").expect("OPERATOR_PWD must be set").as_bytes());
//
//     let concat_str = format!("{}&{}&{}",
//                              &upyun_params.method,
//                              &upyun_params.uri,
//                              &upyun_params.date);
//     let signed_key =
//         hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY,
//                        hasher1.result_str().as_bytes());
//
//     let auth = hmac::sign(&signed_key, concat_str.as_bytes());
//     let b64_encoded_sig = BASE64.encode(auth.as_ref());
//
//     let policy_str = serde_json::to_string(&policy).unwrap();
//     let policy_str_base64 = BASE64.encode(policy_str.as_bytes());
//
//     let sign_str = format!("{}&{}",
//                            policy_str_base64,
//                            env::var("FORM_API_KEY").expect("FORM_API_KEY must be set"));
//
//     let mut hasher2 = Md5::new();
//     hasher2.input(&sign_str.as_bytes());
//
//     UpYunAuth {
//         auth: format!("{} {}:{}",
//                       "UPYUN",
//                       env::var("OPERATOR").expect("OPERATOR must be set"),
//                       b64_encoded_sig),
//         sign: hasher2.result_str(),
//         policy: policy_str_base64,
//     }
// }

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct UpyunFilename {
    pub filename: String,
}

pub async fn upyun_delete_by_filename(in_filename: &String, sudoer: &String, host: &String) -> anyhow::Result<()> {
    debug!("upyun_delete_by_filename -> START: {}", in_filename);
    // let client = reqwest::blocking::Client::new();
    let client = reqwest::Client::new();
    let upyun_filename = UpyunFilename {
        filename: in_filename.clone(),
    };

    debug!("upyun_delete_by_filename -> filename: {}", in_filename);
    debug!("upyun_delete_by_filename -> sudoer: {}", sudoer);
    debug!("upyun_delete_by_filename -> host: {}", host);

    let url = format!("{}/image/upyun_remove", get_ngx_url());
    debug!("upyun_delete_by_filename -> url: {}", url);

    let body = serde_json::to_string::<UpyunFilename>(&upyun_filename).unwrap();
    debug!("upyun_delete_by_filename -> body: {}", body);


    let req = client.post(url)
        .body(body)
        .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
        .header("HtySudoerToken", sudoer)
        .header("HtyHost", host);

    debug!("upyun_delete_by_filename -> req: {:?}", req);


    let resp = req
        .send().await?;

    debug!("upyun_delete_by_filename -> resp: {:?}", resp);
    Ok(())
}