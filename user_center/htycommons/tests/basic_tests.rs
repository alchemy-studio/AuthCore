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

#[test]
fn test_wx_encode_decode() {
    let session_key = "E9+B5t8W2532yB4O8osEDA==";
    let iv = "797GckylrepXN+lwKrq7QA==";
    let encrypted_data = "l69ejoQf+1S1TFx6uJi7m0XqHzygEh9OIjna4lw8SWcgUQ7IVxIMKEFNUSZ4nHb+vDBVeN0E/LFPFkqsf44YhE4yXl9H3eymKDoXzDT0Qalr66zMbN9JU5TlBlP2TKNSfAzl+5Ci36Ysxp3YIrMYT7e3f5G3C4Jzodyn5TPqmnpAlkU202LIZKjFNv4UKUz8kWd9tNuhjDS7HGBfN8hBHn5ixd/y0UtmABJeiMtnKQfsAZh8pL3jTXmBnyn0zkq6Gd6+Mh6LSDZ52NEOSwnjhgRVhLBV9Nhr/hLd3xQ4/rau0idJZbNUVEjT9IJ+RAD695NpkLbHMFNCAK9tSRvdubwM4c/XOiVL+7tl0jd9BfGxJHV3JdeeBNtwnfMdHYIxZkl4podZREDWDgfLbHzllYyvZZdLkFg8p1huxkEGMyzDyEQQX2J51xmKB0htuGhK6wCsfAmuTR5EdKtkM2ORNCepo6pwcKfkQWihc6Dg/wYwy3vNpeFQbMrEEhRSIVM9wE8TnbrLfJLoOaF6YHAsK26U7dUYsAENunMaeCRUlac=";

    let params = WxParams {
        code: None,
        appid: None,
        encrypted_data: Some(encrypted_data.into()),
        iv: Some(iv.into()),
        secret: None,
    };

    let decrypted_data = wx_decode(&params, session_key);
    assert_eq!("{\"openId\":\"o-dnm5ffBPBP4y8bsRDetmE8Gfco\",\"nickName\":\"木逸辰\",\"gender\":1,\"language\":\"zh_CN\",\"city\":\"Haidian\",\"province\":\"Beijing\",\"country\":\"China\",\"avatarUrl\":\"https://wx.qlogo.cn/mmopen/vi_32/VM3Aa9z5YurfbUXica6LcZNcUnTXPs8UJEMicGXicHXuOfqA4bM5zOVia4OoCY8CpHiaGRAgj1HdtQnlh3zLeZKTRTQ/132\",\"unionId\":\"o8vss0XUO8eynm2cjS9MMm7Obx1g\",\"watermark\":{\"timestamp\":1568131106,\"appid\":\"wx0e30fcb4f4fa808a\"}}",
               decrypted_data);

    let wx_user = serde_json::from_str::<WxUser>(decrypted_data.as_str());
    assert_eq!(
        "o-dnm5ffBPBP4y8bsRDetmE8Gfco",
        wx_user.unwrap().openId.unwrap()
    );
}