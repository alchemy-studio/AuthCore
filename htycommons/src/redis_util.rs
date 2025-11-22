use crate::jwt::{jwt_encode_token, jwt_decode_token};
use crate::remove_quote;
use anyhow::anyhow;
use redis::*;
use std::env;
use log::{error, debug};
use crate::web::HtyToken;
use dotenv::dotenv;

pub const WX_ACCESS_TOKEN_PREFIX: &str = "WX_ACCESS_TOKEN_";
pub const WX_JSAPI_TICKET_PREFIX: &str = "WX_JSAPI_TICKET_";
pub const HTY_REDIS_KEY_PREFIX: &str = "HW_";
pub const TOKEN_PREFIX: &str = "T_";
pub const LOGIN_UNION_ID_PREFIX: &str = "LOGIN_UNION_ID_";
pub const ALL_USER_OPENIDS: &str = "ALL_USER_OPENIDS_";
pub const OPENID_INFO: &str = "OPENID_INFO_";
pub const CACHED: &str = "C_";


pub fn get_token_expiration_days() -> anyhow::Result<usize> {
    env::var("EXPIRATION_DAYS")
        .expect("EXPIRATION_DAYS not set!!!")
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse EXPIRATION_DAYS: {}", e))
}

pub fn get_redis_url() -> anyhow::Result<String> {
    let _ = dotenv::dotenv();
    let redis_host = env::var("REDIS_HOST").expect("REDIS_HOST NOT SET!");
    let redis_port = env::var("REDIS_PORT").expect("REDIS_PORT NOT SET!");
    debug!("redis_host -> {:?}", redis_host.clone());
    debug!("redis_port -> {:?}", redis_port.clone());

    let redis_url = remove_quote(&format!("redis://{:?}:{:?}", redis_host, redis_port));
    debug!("redis_url -> {:?}", redis_url.clone());

    Ok(redis_url)
}


pub fn save_token_with_exp_days(token: &HtyToken, expiration_day: usize) -> anyhow::Result<()> {
    if expiration_day <= 0 {
        error!("Failed to save token with error expiration ");
        return Err(anyhow!("save_token_with_expiration_days -> expiration blow zero!"));
    }

    debug!("save_token -> {:?}", token.clone());

    let mut prefix_key = TOKEN_PREFIX.to_string();
    prefix_key.push_str(token.clone().token_id.as_str());

    save_kv_to_redis_with_exp_days(&prefix_key,
                                   &jwt_encode_token(token.clone())?,
                                   expiration_day)
}

pub fn save_kv_to_redis_with_exp_hours(key: &String, val: &String, hours: usize) -> anyhow::Result<()> {
    save_kv_to_redis_with_exp_secs(&key.clone(),
                                   &val.clone(),
                                   hours * 60 * 60)
}

pub fn save_kv_to_redis_with_exp_minutes(key: &String, val: &String, minutes: usize) -> anyhow::Result<()> {
    save_kv_to_redis_with_exp_secs(&key.clone(),
                                   &val.clone(),
                                   minutes * 60)
}


pub fn save_kv_to_redis_with_exp_days(key: &String, val: &String, days: usize) -> anyhow::Result<()> {
    save_kv_to_redis_with_exp_secs(&key.clone(),
                                   &val.clone(),
                                   days * 24 * 60 * 60)
}


pub fn save_kv_to_redis(key: &String, value: &String) -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);
    let redis_url = get_redis_url()?;

    match Client::open(redis_url.clone()) {
        Ok(cli) => match cli.get_connection() {
            Ok(mut redis_connect) => {
                match redis_connect.set(prefix_key, value)
                {
                    Ok(()) => {
                        debug!("save token successfully -> {:?} / {:?}", key, value);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to save token -> {:?} / {:?} / {:?}", key, value, e);
                        Err(anyhow!(e))
                    }
                }
            }
            Err(e) => {
                error!("redis error! -> {:?} / {:?} / {:?}", key, value, e);
                Err(anyhow!(e))
            }
        },
        Err(e) => {
            error!("redis error! -> {:?} / {:?} / {:?}", key, value, e);
            Err(anyhow!(e))
        }
    }
}

pub fn save_kv_to_redis_with_exp_secs(key: &String, value: &String, expiration_sec: usize) -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);
    let redis_url = get_redis_url()?;

    match Client::open(redis_url.clone()) {
        Ok(cli) => match cli.get_connection() {
            Ok(mut redis_connect) => {
                match redis_connect.set_ex(prefix_key, value, expiration_sec as u64)
                {
                    Ok(()) => {
                        debug!("save token with expiration date successfully");
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to save token with expiration -> {:?}", e);
                        Err(anyhow!(e))
                    }
                }
            }
            Err(e) => {
                error!("redis error! -> {:?}", e);
                Err(anyhow!(e))
            }
        },
        Err(e) => {
            error!("redis error! -> {:?}", e);
            Err(anyhow!(e))
        }
    }
}

pub fn get_token_from_redis(token_id: &String) -> anyhow::Result<String> {
    let mut prefix_key = TOKEN_PREFIX.to_string();
    prefix_key.push_str(token_id.as_str());
    debug!("get_token_from_redis() -> key / {:?}", &prefix_key);
    get_value_from_redis(&prefix_key)
}

pub fn get_value_from_redis(key: &String) -> anyhow::Result<String> {
    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);
    debug!("get_value_from_redis() -> key / {:?}", &prefix_key);
    let redis_url = get_redis_url()?;
    let mut redis_conn = Client::open(redis_url.clone())?.get_connection()?;
    let value = redis_conn.get(prefix_key)?;
    debug!("get_value_from_redis() -> token : {:?}", value);
    Ok(value)
}

pub fn get_opt_value_from_redis(key: &String) -> anyhow::Result<Option<String>> {
    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);
    debug!("get_value_from_redis2() -> key / {:?}", &prefix_key);
    let redis_url = get_redis_url()?;
    let mut redis_conn = Client::open(redis_url.clone())?.get_connection()?;
    let value = redis_conn.get(prefix_key)?;
    debug!("get_value_from_redis2() -> token : {:?}", value);
    Ok(value)
}

pub fn is_key_exist_in_redis(key: &String) -> anyhow::Result<bool> {
    let redis_url = get_redis_url()?;

    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);

    let mut redis_conn = Client::open(redis_url.clone())?.get_connection()?;

    let exist = redis_conn.exists(prefix_key)?;

    Ok(exist)
}

pub fn verify_jwt(jwt_token: &String) -> anyhow::Result<()> {
    dotenv().ok();
    //Save token to redis
    let redis_url = get_redis_url()?;
    debug!("verify_jwt -> redis_url / {:?}", redis_url.clone());
    debug!("verify_jwt -> jwt_token to verify: {:?}", jwt_token.clone());

    match jwt_decode_token(jwt_token) {
        Ok(token) => {
            debug!("verify_jwt -> *DECODED* token to verify: {:?}", token);
            debug!("verify_jwt -> *DECODED* token_id : {:?}", token.clone().token_id);
            match get_token_from_redis(&token.token_id) {
                Ok(token_in_redis) => {
                    debug!("verify_jwt -> found token in redis: {:?}", token_in_redis);

                    if token_in_redis == jwt_token.clone() {
                        debug!("verify_jwt -> verify jwt token ok!");
                        Ok(())
                    } else {
                        error!("verify_jwt -> token not match!");
                        Err(anyhow!("verify_jwt -> token not match!"))
                    }
                }
                Err(e) => {
                    error!("verify_jwt -> get_token_from_redis() error! / {:?} / token: {:?}", e, token);
                    Err(anyhow!(e))
                }
            }
        }
        Err(e) => {
            error!("verify_jwt -> jwt_decode_token() error! -> {:?}", e);
            Err(anyhow!(e))
        }
    }
}


pub fn del_from_redis(key: &String) -> anyhow::Result<String> {
    debug!("del_from_redis() -> key : {:?}", key);

    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);

    let redis_url = get_redis_url()?;

    let mut redis_conn = Client::open(redis_url.clone())?.get_connection()?;

    let r = redis_conn.del(prefix_key)?;

    debug!("del_from_redis() -> result : {:?}", r);

    Ok(r)
}



pub fn del_some_from_redis(key: &String) -> anyhow::Result<Option<()>> {
    debug!("del_some_from_redis() -> key : {:?}", key);

    let mut prefix_key = HTY_REDIS_KEY_PREFIX.to_string();
    prefix_key.push_str(key);

    let redis_url = get_redis_url()?;

    let mut redis_conn = Client::open(redis_url.clone())?.get_connection()?;

    let r: () = redis_conn.del(prefix_key)?;

    debug!("del_some_from_redis() -> result : {:?}", r);

    Ok(Some(()))
}

pub fn all_openids_prefix(app_id: &String) -> String {
    format!("{}{}", &ALL_USER_OPENIDS.to_string(), app_id)
}

pub fn openid_info_prefix(openid: &String, appid: &String) -> String {
    format!("{}{}_{}", &OPENID_INFO.to_string(), openid, appid)
}