use crate::common::{current_local_datetime, HtyErr, HtyErrCode};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use log::debug;
use crate::web::{HtyToken, wrap_err};
use crate::common::HtyErrCode::JwtErr;
use crate::{n_hour_later};

#[derive(Debug, Serialize, Deserialize)]
pub struct HtyJwtClaims {
    sub: String,
    exp: usize,
    iat: usize, // issued at, required by microprofile JWT
}


pub fn jwt_key() -> String {
    let _ = dotenv::dotenv();
    env::var("JWT_KEY").expect("JWT_KEY not set⚠️")
}

pub fn jwt_encode_token(token: HtyToken) -> Result<String, HtyErr> {
    let t = n_hour_later(&current_local_datetime(), 10000)
        .map_err(|e| HtyErr {
            code: HtyErrCode::JwtErr,
            reason: Some(format!("Failed to calculate expiration time: {}", e)),
        })?
        .and_utc().timestamp() as usize;
    let claims = HtyJwtClaims {
        sub: serde_json::to_string(&token)
            .map_err(|e| HtyErr {
                code: HtyErrCode::JwtErr,
                reason: Some(format!("Failed to serialize token: {}", e)),
            })?,
        exp: t,
        iat: t,
    };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_key().as_bytes())) {
        Ok(val) => Ok(val),
        Err(err) => Err(HtyErr {
            code: HtyErrCode::JwtErr,
            reason: Some(err.to_string()),
        }),
    }
}

pub fn jwt_decode_token(raw_token: &String) -> Result<HtyToken, HtyErr> {
    let token = raw_token.clone().replacen("\\\"", "", 2);
    //debug(format!("decode token -> {:?}", token).as_str());
    debug!("jwt_decode_token -> token to decode: {:?}", token);

    let mut validation = Validation::new(Algorithm::HS256);
    validation.algorithms = vec![Algorithm::HS256, Algorithm::HS384, Algorithm::HS512];
    validation.validate_exp = false;
    //debug(format!("jwt key -> {:?}", jwt_key()).as_str());
    debug!("jwt_decode_token -> jwt key: {:?}", jwt_key());
    _jwt_decode_claims(&raw_token, &validation)
}


fn _jwt_decode_claims(token: &String, validation: &Validation) -> Result<HtyToken, HtyErr> {
    debug!("_jwt_decode_claims -> token to decode: {:?}", token);
    match decode::<HtyJwtClaims>(&token, &DecodingKey::from_secret(jwt_key().as_bytes()), &validation) {
        Ok(v) => {
            //debug(format!("decoded obj -> {:?}", v).as_str());
            debug!("_jwt_decode_claims -> decoded token:  {:?}", v);
            serde_json::from_str::<HtyToken>(v.claims.sub.as_str())
                .map_err(|e| HtyErr {
                    code: HtyErrCode::JwtErr,
                    reason: Some(format!("Failed to deserialize token: {}", e)),
                })
        }
        Err(err) => Err(wrap_err(JwtErr, Box::from(err))),
    }
}
