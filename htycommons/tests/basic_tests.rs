use htycommons::common::{HtyErr, HtyErrCode, HtyResponse, string_to_date};
use htycommons::web::wrap_hty_err;
use htycommons::logger::{debug, info};
use htycommons::wx::{WxUser, WxParams, wx_decode};

#[test]
pub fn test_datetime() {
    println!("{:?}", string_to_date(&Some("2024-04-16".to_string())));
}

#[test]
pub fn test_hty_err() {
    let err = HtyErr {
        code: HtyErrCode::DbErr,
        reason: Some("The answer is 42".into()),
    };

    info(format!("{:?}", serde_json::to_string(&err).unwrap()).as_str());

    assert_eq!(
        "{\"code\":\"DbErr\",\"reason\":\"The answer is 42\"}",
        serde_json::to_string(&err).unwrap()
    );
}

#[test]
pub fn test_flatten_err() {
    let resp1: HtyResponse<String> = wrap_hty_err(HtyErr { code: HtyErrCode::InternalErr, reason: Some("for test".to_string()) });
    let json1 = serde_json::to_string(&resp1);

    debug(format!("{:?}", json1).as_str());
    assert_eq!("{\"r\":false,\"d\":null,\"e\":\"InternalErr -> for test\"}".to_string(), json1.unwrap());


    let resp2: HtyResponse<String> = wrap_hty_err(
        HtyErr {
            code: HtyErrCode::CommonError,
            reason: Some(HtyErr { code: HtyErrCode::InternalErr, reason: Some("for test".to_string()) }.to_string()),
        });
    let json2 = serde_json::to_string(&resp2);
    debug(format!("{:?}", json2).as_str());
    assert_eq!("{\"r\":false,\"d\":null,\"e\":\"CommonError -> InternalErr -> for test\"}".to_string(), json2.unwrap());
}
