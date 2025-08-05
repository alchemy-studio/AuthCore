use base64::engine::GeneralPurpose;
use base64::{alphabet, engine::general_purpose};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::sql_types::BigInt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::{env, fmt};
use tracing::debug;

pub const APP_STATUS_ACTIVE: &str = "ACTIVE";
pub const APP_STATUS_DELETED: &str = "DELETED";
pub const BASE64_DECODER: GeneralPurpose =
    GeneralPurpose::new(&alphabet::STANDARD, general_purpose::PAD);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct HtyResponse<T: Serialize + DeserializeOwned + Debug + Clone> {
    pub r: bool,
    // result
    pub d: Option<T>,
    // data
    pub e: Option<String>, // err
    pub hty_err: Option<HtyErr>,
}

#[derive(Deserialize, Serialize, Clone, thiserror::Error)]
pub struct HtyErr {
    pub code: HtyErrCode,
    pub reason: Option<String>,
}

impl fmt::Display for HtyErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Debug for HtyErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.code,
            self.reason.clone().get_or_insert("".to_string())
        )
    }
}

impl PartialEq for HtyErr {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.reason == other.reason
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum HtyErrCode {
    DbErr,                // 数据库错误
    InternalErr,          // 内部错误
    CommonError,          // 通用错误
    WebErr,               // 网络错误
    JwtErr,               // JWT错误
    WxErr,                // 微信接口错误
    NullErr,              // 空数据错误
    NotFoundErr,          // 数据不存在错误
    NotEqualErr,          // 数据不相等错误
    AuthenticationFailed, // 认证错误
    ConflictErr,          // 冲突
    TypeErr,              // 类型错误
    DuplicateErr,         // 重复错误
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum TimeUnit {
    DAY,
    HOUR,
    MINUTE,
    SECOND,
}

impl fmt::Display for HtyErrCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl PartialEq for HtyErrCode {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

pub fn env_var(key: &str) -> Option<String> {
    dotenv::dotenv().ok();
    return Some(
        env::var(&key)
            .expect(format!("{:?} not set⚠️", key).as_str())
            .to_string(),
    );
}

#[derive(QueryableByName, Default)]
pub struct CountResult {
    #[diesel(sql_type = BigInt)]
    pub result: i64,
}

pub fn parse_date_time(date_time: &String) -> anyhow::Result<NaiveDateTime> {
    Ok(NaiveDateTime::parse_from_str(
        date_time.as_str(),
        "%Y-%m-%d %H:%M:%S",
    )?)
}

pub fn parse_bool(bool_val: &String) -> anyhow::Result<bool> {
    Ok(bool_val.trim().parse().unwrap())
}

pub fn get_some_from_query_params<T: FromStr + Default>(
    key: &str,
    params: &HashMap<String, String>,
) -> Option<T> {
    if let Some(value) = params.get(key) {
        Some(value.parse::<T>().unwrap_or_default())
    } else {
        None
    }
}

pub fn get_page_and_page_size(params: &HashMap<String, String>) -> (Option<i64>, Option<i64>) {
    let page = params.get("page").and_then(|p| p.parse::<i64>().ok());
    let page_size = params
        .get("page_size")
        .and_then(|ps| ps.parse::<i64>().ok());

    (page, page_size)
}

// https://stackoverflow.com/questions/57707966/how-to-get-timestamp-of-the-current-date-and-time-in-rust
pub fn current_local_datetime() -> NaiveDateTime {
    chrono::offset::Local::now().naive_local()
}

pub fn current_local_date() -> NaiveDateTime {
    // NaiveTime::from_hms_opt(0, 0, 0).unwrap()
    chrono::offset::Local::now()
        .naive_local()
        .date()
        .and_time(NaiveTime::default())
}

pub fn strip_result_vec<T>(in_vec: Vec<anyhow::Result<T>>) -> anyhow::Result<Vec<T>> {
    let mut out_vec = Vec::new();
    for any_item in in_vec {
        out_vec.push(any_item?); // 调用方需要自己保证ok()没问题
    }
    Ok(out_vec)
}

pub fn extract_filename_from_url(url: &String) -> String {
    url.split("/").last().unwrap().to_string()
}

pub fn date_to_string(date: &NaiveDateTime) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn string_to_datetime(datetime: &Option<String>) -> anyhow::Result<Option<NaiveDateTime>> {
    if datetime.is_none() {
        return Ok(None);
    } else {
        Ok(Some(NaiveDateTime::parse_from_str(
            datetime.as_ref().unwrap().as_str(),
            "%Y-%m-%d %H:%M:%S",
        )?))
    }
}

pub fn string_to_date(date: &Option<String>) -> anyhow::Result<Option<NaiveDateTime>> {
    debug!("string_to_date -> {:?}", date);

    if date.is_none() {
        return Ok(None);
    } else {
        Ok(Some(
            NaiveDate::parse_from_str(date.as_ref().unwrap().as_str(), "%Y-%m-%d")?
                .and_time(NaiveTime::from_str("00:00:00")?),
        ))
    }
}

pub fn time_now() -> std::time::Instant {
    // use time::ext::InstantExt;
    std::time::Instant::now()
}
