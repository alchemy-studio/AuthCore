// // use std::env;
//
// use dotenv::dotenv;
// use htyuc::uc_rocket;
//
// use htycommons::logger::{info};
// use log::{debug};
//
// use htycommons::db::get_uc_db_url;
// use htycommons::common::HtyResponse;
// use htycommons::test_scaffold::{do_test, my_assert_eq};
// use htycommons::web::{get_uc_url, random_port};
// use htycommons::wx::{code2session, wx_decode, WxLogin, WxParams, WxSession, WxUser};
// use std::collections::HashMap;
// use std::rc::Rc;
// use htyuc::test_scaffold::HtyucTestScaffold;
// //
// // // https://developers.weixin.qq.com/miniprogram/dev/api-backend/open-api/login/auth.code2Session.html
// // // https://developers.weixin.qq.com/miniprogram/dev/api/open-api/login/wx.login.html
// // // https://developers.weixin.qq.com/doc/offiaccount/Getting_Started/Global_Return_Code.html
// // #[test]
// // fn test_wx_code2session() {
// //     dotenv().ok();
// //     let code = "0932bFkl2z1Y";
// //     let app_id = "mocked_app_id";
// //     let secret = "mocked_secret";
// //     let params = WxParams {
// //         code: Some(code.into()),
// //         appid: Some(app_id.to_string()),
// //         secret: Some(secret.to_string()),
// //         encrypted_data: None,
// //         iv: None,
// //     };
// //
// //     let _ok = async {
// //         let ret = code2session(&params).await.unwrap();
// //         // fixme: need to get correct result(needs session key)
// //         assert_eq!(ret.errcode, Some(40013));
// //     };
// //
// //     ()
// // }
// //
// // #[test]
// // fn test_wx_encode_decode() {
// //     let session_key = "E9+B5t8W2532yB4O8osEDA==";
// //     let iv = "797GckylrepXN+lwKrq7QA==";
// //     let encrypted_data = "l69ejoQf+1S1TFx6uJi7m0XqHzygEh9OIjna4lw8SWcgUQ7IVxIMKEFNUSZ4nHb+vDBVeN0E/LFPFkqsf44YhE4yXl9H3eymKDoXzDT0Qalr66zMbN9JU5TlBlP2TKNSfAzl+5Ci36Ysxp3YIrMYT7e3f5G3C4Jzodyn5TPqmnpAlkU202LIZKjFNv4UKUz8kWd9tNuhjDS7HGBfN8hBHn5ixd/y0UtmABJeiMtnKQfsAZh8pL3jTXmBnyn0zkq6Gd6+Mh6LSDZ52NEOSwnjhgRVhLBV9Nhr/hLd3xQ4/rau0idJZbNUVEjT9IJ+RAD695NpkLbHMFNCAK9tSRvdubwM4c/XOiVL+7tl0jd9BfGxJHV3JdeeBNtwnfMdHYIxZkl4podZREDWDgfLbHzllYyvZZdLkFg8p1huxkEGMyzDyEQQX2J51xmKB0htuGhK6wCsfAmuTR5EdKtkM2ORNCepo6pwcKfkQWihc6Dg/wYwy3vNpeFQbMrEEhRSIVM9wE8TnbrLfJLoOaF6YHAsK26U7dUYsAENunMaeCRUlac=";
// //
// //     let params = WxParams {
// //         code: None,
// //         appid: None,
// //         encrypted_data: Some(encrypted_data.into()),
// //         iv: Some(iv.into()),
// //         secret: None,
// //     };
// //
// //     let decrypted_data = wx_decode(&params, session_key);
// //     info(format!("<><><><><><>{:?}", decrypted_data).as_str());
// //     assert_eq!("{\"openId\":\"o-dnm5ffBPBP4y8bsRDetmE8Gfco\",\"nickName\":\"木逸辰\",\"gender\":1,\"language\":\"zh_CN\",\"city\":\"Haidian\",\"province\":\"Beijing\",\"country\":\"China\",\"avatarUrl\":\"https://wx.qlogo.cn/mmopen/vi_32/VM3Aa9z5YurfbUXica6LcZNcUnTXPs8UJEMicGXicHXuOfqA4bM5zOVia4OoCY8CpHiaGRAgj1HdtQnlh3zLeZKTRTQ/132\",\"unionId\":\"o8vss0XUO8eynm2cjS9MMm7Obx1g\",\"watermark\":{\"timestamp\":1568131106,\"appid\":\"wx0e30fcb4f4fa808a\"}}",
// //                decrypted_data);
// //
// //     let wx_user = serde_json::from_str::<WxUser>(decrypted_data.as_str());
// //     info(format!("wx_user -> {:?}", wx_user).as_str());
// //     assert_eq!(
// //         "o-dnm5ffBPBP4y8bsRDetmE8Gfco",
// //         wx_user.unwrap().openId.unwrap()
// //     );
// // }
// //
// // // HtyResponse { r: false, d: None, e: Some("error sending request for url (https://api.weixin.qq.com/sns/jscode2session?appid=test_app_id&secret=test_secret&js_code=cafebabe&grant_type=authorization_code): error trying to connect: record overflow") }
// // // https://api.weixin.qq.com/sns/jscode2session?appid=test_app_id&secret=test_secret&js_code=cafebabe&grant_type=authorization_code
// // // {"errcode":40013,"errmsg":"invalid appid rid: 608632ff-55d53d7a-230871b3"}
// // #[test]
// // #[ignore]
// // fn test_post_weixin_login() {
// //     let web_test = |_params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //         let login = WxLogin {
// //             code: String::from("cafebabe"),
// //         };
// //         let params = serde_json::to_string(&login).unwrap();
// //         info(format!("{:?}", params).as_str());
// //         let response = client
// //             .post("/api/v1/uc/wx/login/")
// //             .body(params)
// //             .header(ContentType::JSON)
// //             .header(Header::new("host", "localhost"))
// //             .dispatch();
// //
// //         my_assert_eq(response.status(), Status::Ok)?;
// //
// //         let body: HtyResponse<WxSession> =
// //             serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
// //
// //         info(format!("{:?}", body).as_str());
// //
// //         my_assert_eq(40013, body.d.unwrap().errcode.unwrap())?;
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[ignore]
// // #[test]
// // fn test_wx_get_access_token() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //             let token = params["root_token"].clone();
// //
// //             let resp = client
// //                 .get(format!("{}/wx/get_access_token", get_uc_url()))
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "music-room.moicen.com"))
// //                 .header(Header::new("HtySudoerToken", token.clone()))
// //                 .dispatch();
// //             let resp_text = resp.into_string();
// //             let token = serde_json::from_str::<HtyResponse<String>>(resp_text.unwrap().as_str()).unwrap().d.unwrap();
// //             debug!("[WX ACCESS TOKEN] -> {}", token.as_str());
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[ignore]
// // #[test]
// // fn test_wx_get_jsapi_ticket() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //             let token = params["root_token"].clone();
// //
// //             let resp = client
// //                 .get(format!("{}/wx/get_jsapi_ticket", get_uc_url()))
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "wx.moicen.com"))
// //                 .header(Header::new("HtySudoerToken", token.clone()))
// //                 .dispatch();
// //             let resp_text = resp.into_string();
// //             let ticket = serde_json::from_str::<HtyResponse<String>>(resp_text.unwrap().as_str()).unwrap().d.unwrap();
// //             debug!("[WX JSAPI TICKET] -> {}", ticket.as_str());
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
