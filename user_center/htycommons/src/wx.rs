use base64::Engine;
use log::debug;
use crate::common::BASE64_DECODER;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct WxSession {
    pub session_key: Option<String>,
    // from code2session
    pub openid: Option<String>,
    pub unionid: Option<String>,
    // from decryptedData, same with openid, useless
    pub openId: Option<String>,
    // todo : add all the fields
    // https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

// wx_user_info: https://developers.weixin.qq.com/miniprogram/dev/framework/open-ability/signature.html
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct WxUser {
    pub watermark: Option<WxWatermark>,
    pub openId: Option<String>,
    pub unionId: Option<String>,
    pub nickName: Option<String>,
    pub gender: i32,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub avatarUrl: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct WxWatermark {
    pub timestamp: i32,
    pub appid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WxId {
    pub union_id: String,
    pub openid: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ReqWxPushResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub msgid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WxParams {
    pub code: Option<String>,
    // https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
    pub appid: Option<String>,
    pub secret: Option<String>,
    pub encrypted_data: Option<String>,
    pub iv: Option<String>,
}

/// structure for weixin
#[derive(Deserialize, Serialize)]
pub struct WxLogin {
    pub code: String,
}

pub async fn code2session(params: &WxParams) -> anyhow::Result<WxSession> {
    let url = format!("https://api.weixin.qq.com/sns/jscode2session?appid={0}&secret={1}&js_code={2}&grant_type=authorization_code",
                      params.appid.clone().unwrap(),
                      params.secret.clone().unwrap(),
                      params.code.clone().unwrap());

    debug!("code2session -> url -> {}", url);

    let resp = reqwest::get(&url).await?;

    debug!("code2session -> resp -> {:?}", resp);
    let ret = resp.json().await?;

    debug!("code2session -> wx session -> {:?}", ret);
    Ok(ret)
}

pub fn wx_decode(params: &WxParams, session_key: &str) -> String {
    use aes::cipher::{
        block_padding::Pkcs7, generic_array::GenericArray, BlockDecryptMut, KeyIvInit,
    };
    use cbc::Decryptor;
    type Aes128CbcDec = Decryptor<aes::Aes128>;

    let decoded_session_key = BASE64_DECODER.decode(session_key).unwrap();
    let decoded_iv = BASE64_DECODER.decode(params.iv.clone().unwrap()).unwrap();

    let key = GenericArray::clone_from_slice(decoded_session_key.as_slice());
    let iv = GenericArray::clone_from_slice(decoded_iv.as_slice());

    let decoded_encrypted_data = BASE64_DECODER.decode(params.encrypted_data.clone().unwrap()).unwrap();

    let ct_len = decoded_encrypted_data.len();
    let mut buf = vec![0u8; ct_len];

    buf[..ct_len].copy_from_slice(&decoded_encrypted_data[..ct_len]);
    println!("Data buf : [{:?}]", buf);

    let pt = Aes128CbcDec::new(&key, &iv)
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .unwrap();

    std::str::from_utf8(&pt).unwrap().into()
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxAccessToken {
    pub access_token: Option<String>,
    pub expires_in: Option<usize>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxAccessToken1 {
    pub access_token: Option<String>,
    pub expires_in: Option<usize>,
    pub refresh_token: Option<String>,
    pub openid: Option<String>,
    pub scope: Option<String>,
    pub unionid: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxTicket {
    pub ticket: Option<String>,
    pub expires_in: Option<usize>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxAllFollowers {
    pub total: usize,
    pub count: usize,
    pub data: ReqOpenID,
    pub next_openid: String,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqOpenID {
    pub openid: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxFollowerInfo {
    pub subscribe: usize,
    pub openid: String,
    pub language: String,
    pub subscribe_time: usize,
    pub unionid: String,
    pub remark: String,
    pub groupid: usize,
    pub tagid_list: Vec<usize>,
    pub subscribe_scene: String,
    pub qr_scene: usize,
    pub qr_scene_str: String,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxPushMessage<T> {
    pub touser: Option<String>,
    // pub touser_hty_id: Option<String>,
    pub template_id: String,
    pub url: Option<String>,
    pub miniprogram: Option<ReqWxMiniProgram>,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMiniProgram {
    pub appid: String,
    pub pagepath: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageData4KeywordTemplate {
    pub first: ReqWxMessageDataValue,
    pub keyword1: ReqWxMessageDataValue,
    pub keyword2: ReqWxMessageDataValue,
    pub keyword3: ReqWxMessageDataValue,
    pub keyword4: ReqWxMessageDataValue,
    pub remark: ReqWxMessageDataValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageData3KeywordTemplate {
    pub first: ReqWxMessageDataValue,
    pub keyword1: ReqWxMessageDataValue,
    pub keyword2: ReqWxMessageDataValue,
    pub keyword3: ReqWxMessageDataValue,
    pub remark: ReqWxMessageDataValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageData2keywordTemplate {
    pub first: ReqWxMessageDataValue,
    pub keyword1: ReqWxMessageDataValue,
    pub keyword2: ReqWxMessageDataValue,
    pub remark: ReqWxMessageDataValue,
}

// https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Template_Message_Interface.html
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageData3ThingsTemplate {
    pub thing1: ReqWxMessageDataValue,
    pub thing2: ReqWxMessageDataValue,
    pub thing4: ReqWxMessageDataValue,
    pub time3: ReqWxMessageDataValue,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageData3Things2Template {
    pub thing2: ReqWxMessageDataValue,
    pub thing3: ReqWxMessageDataValue,
    pub thing10: ReqWxMessageDataValue,
    pub time9: ReqWxMessageDataValue,
}

// 微信要求的struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqWxMessageDataValue {
    pub value: String,
}

