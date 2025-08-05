// use htycommons::db::get_uc_db_url;
// use htycommons::logger::debug;
// use htycommons::common::{HtyErr, HtyErrCode, HtyResponse};
// use htycommons::test_scaffold::{do_test, my_assert_eq};
// use htycommons::web::random_port;
// use htyuc::uc_rocket;
//
// use std::collections::HashMap;
// use std::rc::Rc;
// use htyuc::test_scaffold::HtyucTestScaffold;
//
// #[test]
// fn test_get_upyun_token() {
//     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
//         let data = String::from("PUT&/bucket/client_37ascii&1528531186");
//
//         let token = params["root_token"].clone();
//
//         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
//         let resp = client
//             .post("/api/v1/uc/upyun/upyun_token")
//             .header(Header::new("HtySudoerToken", token))
//             .body(data)
//             .dispatch();
//         let res =
//             serde_json::from_str::<HtyResponse<String>>(resp.into_string().unwrap().as_str())?;
//         debug(format!("{:?}", res).as_str());
//         my_assert_eq(
//             "UPYUN moicen:vn2LKkUHcAixN+bG0BfrQroMC9w=",
//             res.d
//                 .ok_or(HtyErr {
//                     code: HtyErrCode::NullErr,
//                     reason: Some("empty app_id".to_string()),
//                 })?
//                 .as_str(),
//         )?;
//
//         Ok(())
//     };
//     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// }
