// use log::info;
//
// use htycommons::{uuid};
// use htyuc::models::{ReqHtyUser, ReqUserAppInfo};
//
// // $ cargo test -- --test-threads=1
// // https://doc.rust-lang.org/book/ch11-02-running-tests.html
//
// fn create_mocked_user_with_info(
//     app_id: &str,
//     hty_id: Option<String>,
// ) -> ReqHtyUserWithInfos {
//     let user = ReqHtyUser {
//         hty_id: hty_id.clone(),
//         union_id: None,
//         enabled: None,
//         created_at: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     };
//
//     let info = ReqUserAppInfo {
//         id: Some(uuid()),
//         app_id: Some(String::from(app_id)),
//         hty_id: hty_id.clone(),
//         openid: None,
//         is_registered: false,
//         username: None,
//         password: None,
//         roles: None,
//         meta: None,
//         created_at: None,
//         teacher_info: None,
//         student_info: None,
//         reject_reason: None,
//         needs_refresh: None
//     };
//
//     (user, info)
// }
//
// #[test]
// fn test_user_with_info_struct() {
//     info!(
//             "{:?}",
//             serde_json::to_string::<ReqHtyUserWithInfos>(&create_mocked_user_with_info(
//                 uuid().as_str(),
//                 None,
//             ))
//                 .unwrap()
//
//     );
// }
//
// //
// // #[test]
// // fn test_create_hty_resource() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //         let token = params["root_token"].clone();
// //
// //         let mut in_params = params.clone();
// //         in_params.insert("username".to_string(), "test_user3".to_string());
// //         in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //         verify_user(&in_params, &client, &token)?;
// //
// //         let resp1 = client
// //             .get(format!("{}/login2_with_unionid", get_uc_url()))
// //             .header(Header::new("UnionId", "test_union"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let encoded_token = serde_json::from_str::<HtyResponse<String>>(
// //             resp1.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         let out_resource = ReqHtyResource {
// //             app_id: None,
// //             created_at: None,
// //             created_by: None,
// //             filename: None,
// //             hty_resource_id: None,
// //             res_type: None,
// //             url: Some("test_url".to_string()),
// //             task_id: None,
// //         };
// //
// //         let json_params = serde_json::to_string::<ReqHtyResource>(&out_resource)?;
// //
// //         let resp2 = client
// //             .post(format!("{}/create_hty_resource", get_uc_url()))
// //             .body(json_params)
// //             .header(ContentType::JSON)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(Header::new("Authorization", encoded_token))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp2_decoded = serde_json::from_str::<HtyResponse<String>>(
// //             resp2
// //                 .into_string()
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::CommonError,
// //                     reason: Some("resp error".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //         my_assert_eq(true, resp2_decoded.r)?;
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_users_by_domain() {
// //     // web test
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //         let mut in_params = params.clone();
// //         in_params.insert("username".to_string(), "test_user1".to_string());
// //         in_params.insert("hty_id".to_string(), in_params["hty_id1"].clone());
// //         let token = params["root_token"].clone();
// //         verify_user(&in_params, &client, &token)?;
// //
// //         let resp = client
// //             .get(format!("{}/find_users_by_domain", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp = serde_json::from_str::<HtyResponse<Vec<ReqHtyUser>>>(
// //             resp.into_string().unwrap().as_str(),
// //         )?;
// //         let users = resp.d;
// //
// //         my_assert_eq(3, users.unwrap().len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_user_with_info() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let username = uuid();
// //
// //         let token = params["root_token"].clone();
// //
// //         let out_user = ReqHtyUser {
// //             hty_id: None,
// //             union_id: None,
// //             enabled: None,
// //             created_at: None,
// //             real_name: None,
// //         };
// //
// //         let out_info = ReqUserAppInfo {
// //             id: None,
// //             app_id: Some(params["app_id"].clone()),
// //             hty_id: None,
// //             openid: None,
// //             is_registered: false,
// //             username: Some(String::from("test")),
// //             password: None,
// //             roles: None,
// //         };
// //
// //         let out_info_duplicate_username = ReqUserAppInfo {
// //             id: None,
// //             app_id: Some(params["app_id"].clone()),
// //             hty_id: None,
// //             openid: None,
// //             is_registered: false,
// //             username: Some(String::from("test")),
// //             password: None,
// //             roles: None,
// //         };
// //
// //         let out_info_without_app_id = ReqUserAppInfo {
// //             id: None,
// //             app_id: None,
// //             hty_id: None,
// //             openid: None,
// //             is_registered: false,
// //             username: None,
// //             password: None,
// //             roles: None,
// //         };
// //
// //         let out_info_with_invalid_app_id = ReqUserAppInfo {
// //             id: None,
// //             app_id: Some("fake app id".to_string()),
// //             hty_id: None,
// //             openid: None,
// //             is_registered: false,
// //             username: None,
// //             password: None,
// //             roles: None,
// //         };
// //
// //         let json_params_happy_flow =
// //             serde_json::to_string::<ReqHtyUserWithInfos>(&(out_user.clone(), out_info));
// //
// //         let json_params_duplicate_username = serde_json::to_string::<ReqHtyUserWithInfos>(
// //             &(out_user.clone(), out_info_duplicate_username),
// //         );
// //
// //         let json_params_invalid_appid = serde_json::to_string::<ReqHtyUserWithInfos>(&(
// //             out_user.clone(),
// //             out_info_with_invalid_app_id,
// //         ));
// //
// //         let json_params_without_appid = serde_json::to_string::<ReqHtyUserWithInfos>(&(
// //             out_user.clone(),
// //             out_info_without_app_id,
// //         ));
// //
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let resp_happy_flow = client
// //             .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //             .body(json_params_happy_flow.unwrap())
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             // .header(Header::new("HtyHost", "mocked_app"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp_duplicate_username = client
// //             .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //             .body(json_params_duplicate_username.unwrap())
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             // .header(Header::new("HtyHost", "mocked_app"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp_invalid_appid = client
// //             .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //             .body(json_params_invalid_appid.unwrap())
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             // .header(Header::new("HtyHost", "mocked_app"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp_without_appid = client
// //             .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //             .body(json_params_without_appid.unwrap())
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             // .header(Header::new("HtyHost", "mocked_app"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let res_happy = serde_json::from_str::<
// //             HtyResponse<ReqHtyUserWithInfos>,
// //         >(resp_happy_flow.into_string().unwrap().as_str())?;
// //
// //         let res_duplicate_username =
// //             serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //                 resp_duplicate_username.into_string().unwrap().as_str(),
// //             )?;
// //
// //         let res_invalid_appid = serde_json::from_str::<
// //             HtyResponse<ReqHtyUserWithInfos>,
// //         >(resp_invalid_appid.into_string().unwrap().as_str())?;
// //
// //         let res_without_appid = serde_json::from_str::<
// //             HtyResponse<ReqHtyUserWithInfos>,
// //         >(resp_without_appid.into_string().unwrap().as_str())?;
// //
// //         my_assert_eq(true, res_happy.r)?;
// //
// //         my_assert_eq(false, res_duplicate_username.r)?;
// //
// //         my_assert_eq(false, res_invalid_appid.r)?;
// //
// //         my_assert_eq(true, res_without_appid.r)?;
// //
// //         let (hty_user_happy, _user_app_infos_happy) = res_happy.d.unwrap();
// //         let (_hty_user_without_appid, _user_app_infos_without_appid) = res_without_appid.d.unwrap();
// //         my_assert_eq(
// //             "test_app_id",
// //             _user_app_infos_without_appid
// //                 .unwrap()
// //                 .get(0)
// //                 .unwrap()
// //                 .app_id
// //                 .as_ref()
// //                 .unwrap(),
// //         )?;
// //
// //         let mut in_params_happy = HashMap::new();
// //         in_params_happy.insert("hty_id".to_string(), hty_user_happy.hty_id.unwrap().clone());
// //         in_params_happy.insert("username".to_string(), username.clone());
// //         in_params_happy.insert("user_role".to_string(), "TEST".to_string());
// //         verify_user(&in_params_happy, &client, &token)?;
// //         verify_info(&in_params_happy, &client, &token)?;
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_or_update_userinfo_with_roles() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>{
// //         let token = params["root_token"].clone();
// //
// //         let role1 = ReqHtyRole {
// //             hty_role_id: Some(params["hty_role1_id"].clone()),
// //             user_app_info_id: None,
// //             app_ids: None,
// //             role_name: None,
// //             role_desc: None,
// //             role_status: None,
// //             labels: None,
// //             actions: None,
// //         };
// //
// //         let role2 = ReqHtyRole {
// //             hty_role_id: Some(params["hty_role2_id"].clone()),
// //             user_app_info_id: None,
// //             app_ids: None,
// //             role_name: None,
// //             role_desc: None,
// //             role_status: None,
// //             labels: None,
// //             actions: None,
// //         };
// //
// //         let req_user_app_info_delete = ReqUserAppInfo {
// //             id: Some(params["created_info1"].clone()),
// //             app_id: None,
// //             hty_id: Some(params["hty_id1"].clone()),
// //             openid: None,
// //             is_registered: false,
// //             username: None,
// //             password: None,
// //             roles: Some(vec![role1.clone(), role2.clone()]),
// //         };
// //
// //
// //         let req_user_app_info_update = ReqUserAppInfo {
// //             id: Some(params["created_info1"].clone()),
// //             app_id: None,
// //             hty_id: Some(params["hty_id1"].clone()),
// //             openid: None,
// //             is_registered: false,
// //             username: None,
// //             password: None,
// //             roles: Some(vec![role1]),
// //         };
// //
// //         let json_params_delete = serde_json::to_string::<ReqUserAppInfo>(&req_user_app_info_delete)?;
// //
// //         let json_params_update = serde_json::to_string::<ReqUserAppInfo>(&req_user_app_info_update)?;
// //
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         client
// //             .post(format!("{}/create_or_update_userinfo_with_roles", get_uc_url()))
// //             .body(json_params_delete)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //         my_assert_eq(UserInfoRole::verify_exist_by_user_info_id_and_role_id(&params["created_info1"].clone(), &params["hty_role2_id"], &conn)?, true)?;
// //
// //
// //         client
// //             .post(format!("{}/create_or_update_userinfo_with_roles", get_uc_url()))
// //             .body(json_params_update)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //         my_assert_eq(UserInfoRole::verify_exist_by_user_info_id_and_role_id(&params["created_info1"].clone(), &params["hty_role2_id"], &conn)?, false)?;
// //
// //
// //         Ok(())
// //     };
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_user_with_null_info() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         // web test
// //         {
// //             let username = uuid();
// //             let token = params["root_token"].clone();
// //
// //             let out_user = ReqHtyUser {
// //                 hty_id: None,
// //                 union_id: None,
// //                 enabled: None,
// //                 created_at: None,
// //                 real_name: None,
// //             };
// //
// //             let non: Option<ReqUserAppInfo> = None;
// //
// //             let json_params =
// //                 serde_json::to_string::<(ReqHtyUser, Option<ReqUserAppInfo>)>(&(out_user, non));
// //
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let resp = client
// //                 .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //                 .body(json_params.unwrap())
// //                 .header(Header::new("HtySudoerToken", token.clone()))
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             let resp = serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //                 resp.into_string().unwrap().as_str(),
// //             )?;
// //
// //             my_assert_eq(true, resp.r)?;
// //
// //             let (hty_user, _user_app_infos) = resp.d.unwrap();
// //
// //             let mut in_params = HashMap::new();
// //             in_params.insert("hty_id".to_string(), hty_user.hty_id.unwrap().clone());
// //             in_params.insert("username".to_string(), username.clone());
// //             in_params.insert("user_role".to_string(), "test_role.to_string()".to_string());
// //             verify_user(&in_params, &client, &token)?;
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_update_user_with_info() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user1".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id1"].clone());
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //             let token = params["root_token"].clone();
// //
// //             // update user
// //             {
// //                 let (out_user, out_info) = create_mocked_user_with_info(
// //                     in_params["app_id"].as_str(),
// //                     Some(in_params["hty_id"].clone()),
// //                 );
// //
// //                 let json_params =
// //                     serde_json::to_string::<ReqHtyUserWithInfos>(&(out_user, out_info))?;
// //
// //                 let resp = client
// //                     .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //                     .body(json_params)
// //                     .header(ContentType::JSON)
// //                     .header(Header::new("HtySudoerToken", token.clone()))
// //                     .header(Header::new("HtyHost", "mocked_app"))
// //                     .dispatch();
// //
// //                 my_assert_eq(
// //                     true,
// //                     serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //                         resp.into_string().unwrap().as_str())?.r,
// //                 )?;
// //             }
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //             verify_info(&in_params, &client, &params["root_token"].clone())?;
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_update_user_with_null_info() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user1".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id1"].clone());
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //             let token = params["root_token"].clone();
// //
// //             // update user
// //             {
// //                 let (out_user, _) = create_mocked_user_with_info(
// //                     in_params["app_id"].as_str(),
// //                     Some(in_params["hty_id1"].clone()),
// //                 );
// //
// //                 let non: Option<ReqUserAppInfo> = None;
// //
// //                 let json_params = serde_json::to_string::<(Option<ReqHtyUser>, Option<ReqUserAppInfo>)>(
// //                     &(Some(out_user.clone()), non),
// //                 );
// //                 let resp = client
// //                     .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //                     .body(json_params.unwrap())
// //                     .header(ContentType::JSON)
// //                     .header(Header::new("HtySudoerToken", token.clone()))
// //                     .header(Header::new("HtyHost", "mocked_app"))
// //                     .dispatch();
// //
// //                 my_assert_eq(
// //                     true,
// //                     serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //                         resp.into_string().unwrap().as_str()
// //                     )?.r,
// //                 )?;
// //             }
// //
// //             verify_user(&in_params, &client, &token)?;
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_update_null_user_with_info() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             let token = params["root_token"].clone();
// //
// //             in_params.insert("username".to_string(), "test_user1".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id1"].clone());
// //
// //             verify_user(&in_params, &client, &token)?;
// //
// //             // update user
// //             {
// //                 let (_, out_info) = create_mocked_user_with_info(
// //                     in_params["app_id"].as_str(),
// //                     Some(in_params["hty_id1"].clone()),
// //                 );
// //
// //                 let non: Option<ReqHtyUser> = None;
// //
// //                 let json_params = serde_json::to_string::<(Option<ReqHtyUser>, Option<ReqUserAppInfo>)>(
// //                     &(non, Some(out_info.clone())),
// //                 );
// //
// //                 let resp = client
// //                     .post(format!("{}/create_or_update_user_with_info", get_uc_url()))
// //                     .body(json_params.unwrap())
// //                     .header(ContentType::JSON)
// //                     .header(Header::new("HtySudoerToken", token.clone()))
// //                     .header(Header::new("HtyHost", "mocked_app"))
// //                     .dispatch();
// //
// //                 my_assert_eq(
// //                     true,
// //                     serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //                         resp.into_string().unwrap().as_str()
// //                     )?.r,
// //                 )?;
// //             }
// //
// //             verify_info(&in_params, &client, &token)?;
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_sudo() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user3".to_string());
// //             in_params.insert("password".to_string(), "testpass".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //             let req_login = ReqLogin {
// //                 username: Some(in_params["username"].clone()),
// //                 password: Some(in_params["password"].clone()),
// //             };
// //
// //             let json_params = serde_json::to_string::<ReqLogin>(&req_login)?;
// //
// //             let resp = client
// //                 .post(format!("{}/login_with_password", get_uc_url()))
// //                 .body(json_params)
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             let resp_string = resp.into_string();
// //             let jwt = serde_json::from_str::<HtyResponse<String>>(resp_string.unwrap().as_str());
// //             let login_token = jwt_decode_token(jwt.unwrap().d.unwrap()).unwrap();
// //             let login_token_jwt = jwt_encode_token(login_token)?;
// //             let resp = client
// //                 .post(format!("{}/sudo", get_uc_url()))
// //                 .header(Header::new("Authorization", login_token_jwt))
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             let resp = resp.into_string().clone().unwrap();
// //             jwt_decode_token(serde_json::from_str::<HtyResponse<String>>(resp.clone().as_str()).unwrap().d.unwrap())?;
// //
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_login_with_password() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user3".to_string());
// //             in_params.insert("password".to_string(), "testpass".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //             let req_login = ReqLogin {
// //                 username: Some(in_params["username"].clone()),
// //                 password: Some(in_params["password"].clone()),
// //             };
// //
// //             let json_params = serde_json::to_string::<ReqLogin>(&req_login)?;
// //             let resp = client
// //                 .post(format!("{}/login_with_password", get_uc_url()))
// //                 .body(json_params)
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //             let resp_text = resp.into_string();
// //             let token = jwt_decode_token(serde_json::from_str::<HtyResponse<String>>(resp_text.unwrap().as_str()).unwrap().d.unwrap());
// //             assert_eq!(in_params["hty_id"], token.unwrap().hty_id.unwrap());
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_login2_with_unionid() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>         // web test
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user3".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //             let token = params["root_token"].clone();
// //
// //             verify_user(&in_params, &client, &token)?;
// //
// //             let resp_user_exist = client
// //                 .get(format!("{}/login2_with_unionid", get_uc_url()))
// //                 .header(Header::new("UnionId", "test_union"))
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             let first_time_user_unionid = uuid().to_owned();
// //             let resp_first_time_user = client
// //                 .get(format!("{}/login2_with_unionid", get_uc_url()))
// //                 .header(Header::new("UnionId", first_time_user_unionid.clone()))
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             let result_user_exist = resp_user_exist.into_string().clone().unwrap();
// //             let result_first_time_user = resp_first_time_user.into_string().clone().unwrap();
// //             let token_user_exist = jwt_decode_token(serde_json::from_str::<HtyResponse<String>>(result_user_exist.clone().as_str()).unwrap().d.unwrap())?;
// //             let token_first_time_user = jwt_decode_token(serde_json::from_str::<HtyResponse<String>>(result_first_time_user.clone().as_str()).unwrap().d.unwrap())?;
// //             let uc_pool = db::pool(&get_uc_db_url());
// //             let conn = uc_pool.get()?;
// //
// //             my_assert_eq(token_first_time_user.hty_id.unwrap(), HtyUser::find_by_union_id(&first_time_user_unionid, &conn)?.hty_id)?;
// //
// //             let res = HtyUser::verify_exist_by_union_id(&String::from(first_time_user_unionid.clone().as_str()), &conn)?;
// //             HtyUser::find_by_union_id(&String::from(first_time_user_unionid.clone().as_str()), &conn)?;
// //
// //             my_assert_eq(true, res)?;
// //             my_assert_eq(in_params["hty_id3"].clone(), token_user_exist.hty_id.unwrap())?;
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_users_with_info_by_role() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>{
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_users_with_info_by_role/{}", get_uc_url(), String::from("TEACHER")))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let data = serde_json::from_str::<HtyResponse<Vec<ReqHtyUserWithInfos>>>(
// //             resp.into_string()
// //                 .clone()
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::NullErr,
// //                     reason: Some("empty resp".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //         my_assert_eq(data.d.unwrap().len(), 0)?;
// //         Ok(())
// //     };
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_user_with_info_by_token() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //         let first_time_user_unionid = uuid().to_owned();
// //         let resp_first_time_user = client
// //             .get(format!("{}/login2_with_unionid", get_uc_url()))
// //             .header(Header::new("UnionId", first_time_user_unionid.to_owned()))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let result_first_time_user = resp_first_time_user.into_string().clone().ok_or(HtyErr {
// //             code: HtyErrCode::NullErr,
// //             reason: Some("empty resp".to_string()),
// //         })?;
// //
// //         let token_first_time_user =
// //             serde_json::from_str::<HtyResponse<String>>(result_first_time_user.as_str())?
// //                 .d
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::NullErr,
// //                     reason: Some("empty result_first_time_user".to_string()),
// //                 })?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_user_with_info_by_token", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(Header::new("Authorization", token_first_time_user))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let data = serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //             resp.into_string()
// //                 .clone()
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::NullErr,
// //                     reason: Some("empty resp".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //
// //         let (user, info) = data.d.ok_or(HtyErr {
// //             code: HtyErrCode::NullErr,
// //             reason: Some("empty data".to_string()),
// //         })?;
// //
// //         my_assert_eq(
// //             first_time_user_unionid.as_str(),
// //             user.union_id
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::NullErr,
// //                     reason: Some("empty union_id".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //         my_assert_eq(
// //             "test_app_id",
// //             info.app_id
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::NullErr,
// //                     reason: Some("empty app_id".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // fn verify_user(
// //     in_params: &HashMap<String, String>,
// //     client: &Client,
// //     root_token: &String,
// // ) -> anyhow::Result<()> {
// //     let req = format!(
// //         "{}/find_user_with_info_by_id/{}",
// //         get_uc_url(),
// //         in_params["hty_id"].clone()
// //     );
// //
// //     let resp = client
// //         .get(req)
// //         .header(ContentType::JSON)
// //         .header(Header::new("HtySudoerToken", root_token.clone()))
// //         .dispatch();
// //
// //     let data = serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //         resp.into_string().unwrap().as_str(),
// //     )?;
// //
// //
// //     let ok = my_assert_eq(true, data.r)?;
// //     Ok(ok)
// // }
// //
// // fn verify_info(
// //     in_params: &HashMap<String, String>,
// //     client: &Client,
// //     root_token: &String,
// // ) -> anyhow::Result<()> {
// //     let req = format!(
// //         "{}/find_user_with_info_by_id/{}",
// //         get_uc_url(),
// //         in_params["hty_id"].clone()
// //     );
// //
// //     let resp = client
// //         .get(req)
// //         .header(ContentType::JSON)
// //         .header(Header::new("HtySudoerToken", root_token.clone()))
// //         .dispatch();
// //
// //     let data = serde_json::from_str::<HtyResponse<ReqHtyUserWithInfos>>(
// //         resp.into_string().unwrap().as_str(),
// //     )?;
// //     my_assert_eq(true, data.r)?;
// //     let info = data.d.unwrap().1.unwrap()[0].clone();
// //     my_assert_eq(
// //         in_params["hty_id"].clone(),
// //         info.hty_id.unwrap().to_string(),
// //     )?;
// //
// //     Ok(())
// // }
// //
// // #[test]
// // fn test_find_hty_resources_by_task_id() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let mut in_params = params.clone();
// //         in_params.insert("username".to_string(), "test_user3".to_string());
// //         in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //         verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //         let resp1 = client
// //             .get(format!("{}/login2_with_unionid", get_uc_url()))
// //             .header(Header::new("UnionId", "test_union"))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let encoded_token = serde_json::from_str::<HtyResponse<String>>(
// //             resp1.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         let test_find_hty_resources_by_task_id = uuid();
// //         let out_resource = ReqHtyResource {
// //             app_id: None,
// //             created_at: None,
// //             //created_by: Some(in_params["hty_id3"].clone()),
// //             created_by: None,
// //             filename: None,
// //             hty_resource_id: None,
// //             res_type: None,
// //             url: Some("test_url".to_string()),
// //             task_id: Some(test_find_hty_resources_by_task_id.clone()),
// //         };
// //
// //         let json_params = serde_json::to_string::<ReqHtyResource>(&out_resource)?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp2 = client
// //             .post(format!("{}/create_hty_resource", get_uc_url()))
// //             .body(json_params)
// //             .header(ContentType::JSON)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(Header::new("Authorization", encoded_token))
// //             .header(Header::new("HtyHost", "mocked_app"))
// //             .dispatch();
// //
// //         let resp2_decoded = serde_json::from_str::<HtyResponse<String>>(
// //             resp2
// //                 .into_string()
// //                 .ok_or(HtyErr {
// //                     code: HtyErrCode::CommonError,
// //                     reason: Some("resp error".to_string()),
// //                 })?
// //                 .as_str(),
// //         )?;
// //         my_assert_eq(true, resp2_decoded.r)?;
// //
// //         /////////////////////////////////////////////
// //
// //         let req = format!(
// //             "{}/find_hty_resources_by_task_id/{}",
// //             get_uc_url(),
// //             test_find_hty_resources_by_task_id
// //         );
// //
// //         let resp = client
// //             .get(req)
// //             .header(ContentType::JSON)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let data = serde_json::from_str::<HtyResponse<Option<Vec<ReqHtyResource>>>>(
// //             resp.into_string().unwrap().as_str(),
// //         )?;
// //
// //         my_assert_eq(true, data.r)?;
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_all_roles() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_all_roles", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let roles = serde_json::from_str::<HtyResponse<Vec<ReqHtyRole>>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(7, roles.len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_all_users() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_all_users", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let users = serde_json::from_str::<HtyResponse<Vec<ReqHtyUserWithInfos>>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_not_eq(0, users.len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_all_apps() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_all_apps_with_roles", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let apps = serde_json::from_str::<HtyResponse<Vec<ReqHtyApp>>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //
// //         my_assert_not_eq(0, apps.len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_all_actions() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_all_actions", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let roles = serde_json::from_str::<HtyResponse<Vec<ReqHtyAction>>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(3, roles.len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// //
// // #[test]
// // fn test_find_all_labels() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!("{}/find_all_labels", get_uc_url()))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let labels = serde_json::from_str::<HtyResponse<Vec<ReqHtyLabel>>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(5, labels.len())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_or_update_apps_with_roles() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let req_hty_app_create = ReqHtyApp {
// //             app_id: Some(String::from("test_app_id_create")),
// //             wx_id: None,
// //             secret: Some("secret_create".to_string()),
// //             domain: Some("domain".to_string()),
// //             app_desc: Some("app_desc".to_string()),
// //             app_status: Some("ACTIVE".to_string()),
// //             role_ids: None,
// //             roles: None,
// //             tags: None,
// //             pubkey: None,
// //             privkey: None
// //         };
// //
// //         let req_hty_app_update = ReqHtyApp {
// //             app_id: Some(String::from("test_app_id_create")),
// //             wx_id: None,
// //             secret: Some("secret_update".to_string()),
// //             domain: Some("domain".to_string()),
// //             app_desc: Some("app_desc".to_string()),
// //             app_status: Some("ACTIVE".to_string()),
// //             role_ids: Some(vec![params["hty_role1_id"].clone()]),
// //             roles: None,
// //             tags: None,
// //             pubkey: None,
// //             privkey: None
// //         };
// //
// //         let json_params_create = serde_json::to_string::<ReqHtyApp>(&req_hty_app_create)?;
// //
// //         let json_params_update = serde_json::to_string::<ReqHtyApp>(&req_hty_app_update)?;
// //
// //         client
// //             .post(format!("{}/create_or_update_apps_with_roles", get_uc_url()))
// //             .body(json_params_create)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let res = HtyApp::find_by_domain(&"domain".to_string(), &conn)?;
// //         my_assert_eq(res.clone().secret, "secret_create".to_string())?;
// //
// //         client
// //             .post(format!("{}/create_or_update_apps_with_roles", get_uc_url()))
// //             .body(json_params_update)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let res = HtyApp::find_by_domain(&"domain".to_string(), &conn)?;
// //         my_assert_eq(res.clone().secret, "secret_update".to_string())?;
// //
// //
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_delete_app_by_id() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         client
// //             .post(format!(
// //                 "{}/delete_app_by_id/{}",
// //                 get_uc_url(),
// //                 "test_app_id".to_string()
// //             ))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //         let updated = HtyApp::find_by_id(&"test_app_id".to_string(), &conn)?;
// //         my_assert_eq(updated.app_status, APP_STATUS_DELETED.clone().to_string())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_find_app_with_roles() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let resp = client
// //             .get(format!(
// //                 "{}/find_app_with_roles/{}",
// //                 get_uc_url(),
// //                 "test_app_id".to_string()
// //             ))
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .dispatch();
// //
// //         let req_app = serde_json::from_str::<HtyResponse<ReqHtyApp>>(
// //             resp.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         let role_name = req_app.roles.unwrap()[0].clone().role_name;
// //
// //         my_assert_eq(role_name, "test_read_role".to_string())?;
// //
// //         Ok(())
// //     };
// //
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_or_update_actions() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let in_label1 = ReqHtyLabel {
// //             hty_label_id: Some(params["hty_label1_id"].clone()),
// //             label_name: None,
// //             label_desc: None,
// //             label_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             actions: None,
// //         };
// //
// //         let req_hty_action_create = ReqHtyAction {
// //             hty_action_id: None,
// //             action_name: Some("test_action_1".to_string()),
// //             action_desc: None,
// //             action_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             labels: Some(vec![in_label1.clone()]),
// //         };
// //
// //         let json_params_create = serde_json::to_string::<ReqHtyAction>(&req_hty_action_create)?;
// //
// //         let resp_create = client
// //             .post(format!("{}/create_or_update_actions", get_uc_url()))
// //             .body(json_params_create)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let action_new = serde_json::from_str::<HtyResponse<ReqHtyAction>>(
// //             resp_create.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(
// //             ActionLabel::verify_exist_by_action_id_and_label_id(
// //                 &action_new.hty_action_id.clone().unwrap(),
// //                 &params["hty_label1_id"].clone(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             true,
// //         )?;
// //
// //         let req_hty_action_update = ReqHtyAction {
// //             hty_action_id: action_new.clone().hty_action_id,
// //             action_name: Some("test_action_1".to_string()),
// //             action_desc: None,
// //             action_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             labels: None,
// //         };
// //
// //         let json_params_update = serde_json::to_string::<ReqHtyAction>(&req_hty_action_update)?;
// //
// //         let resp_update = client
// //             .post(format!("{}/create_or_update_actions", get_uc_url()))
// //             .body(json_params_update)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         serde_json::from_str::<HtyResponse<ReqHtyAction>>(
// //             resp_update.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(
// //             ActionLabel::verify_exist_by_action_id_and_label_id(
// //                 &action_new.hty_action_id.clone().unwrap(),
// //                 &params["hty_label1_id"].clone(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             false,
// //         )?;
// //
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_or_update_roles() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let in_label1 = ReqHtyLabel {
// //             hty_label_id: Some(params["hty_label1_id"].clone()),
// //             label_name: None,
// //             label_desc: None,
// //             label_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             actions: None,
// //         };
// //
// //         let req_hty_role_create = ReqHtyRole {
// //             hty_role_id: None,
// //             user_app_info_id: None,
// //             app_ids: None,
// //             role_name: Some("test_role_1".to_string()),
// //             role_desc: None,
// //             role_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             labels: Some(vec![in_label1]),
// //             actions: None,
// //         };
// //
// //         let json_params_create = serde_json::to_string::<ReqHtyRole>(&req_hty_role_create)?;
// //
// //         let resp_create = client
// //             .post(format!("{}/create_or_update_roles", get_uc_url()))
// //             .body(json_params_create)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let roles_new = serde_json::from_str::<HtyResponse<ReqHtyRole>>(
// //             resp_create.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(
// //             RoleLabel::verify_exist_by_role_id_and_label_id(
// //                 &roles_new.hty_role_id.clone().unwrap(),
// //                 &params["hty_label1_id"].clone(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             true,
// //         )?;
// //
// //         let req_hty_role_update = ReqHtyRole {
// //             hty_role_id: roles_new.clone().hty_role_id,
// //             user_app_info_id: None,
// //             app_ids: None,
// //             role_name: Some("test_role_1".to_string()),
// //             role_desc: None,
// //             role_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             labels: None,
// //             actions: None,
// //         };
// //
// //         let json_params_update = serde_json::to_string::<ReqHtyRole>(&req_hty_role_update)?;
// //
// //         let resp_update = client
// //             .post(format!("{}/create_or_update_roles", get_uc_url()))
// //             .body(json_params_update)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         serde_json::from_str::<HtyResponse<ReqHtyRole>>(
// //             resp_update.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //
// //         my_assert_eq(
// //             RoleLabel::verify_exist_by_role_id_and_label_id(
// //                 &roles_new.hty_role_id.clone().unwrap(),
// //                 &params["hty_label1_id"].clone(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             false,
// //         )?;
// //
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_create_or_update_labels() {
// //     let my_test = |params: HashMap<String, String>| -> anyhow::Result<()> {
// //         let uc_pool = db::pool(&get_uc_db_url());
// //         let conn = uc_pool.get()?;
// //
// //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //         let token = params["root_token"].clone();
// //
// //         let in_action1 = ReqHtyAction {
// //             hty_action_id: Some(params["hty_action1_id"].clone()),
// //             action_name: None,
// //             action_desc: None,
// //             action_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             labels: None,
// //         };
// //
// //         let req_hty_label_create = ReqHtyLabel {
// //             hty_label_id: None,
// //             label_name: Some("test_label_1".to_string()),
// //             label_desc: None,
// //             label_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             actions: Some(vec![in_action1.clone()]),
// //         };
// //
// //         let json_params_create = serde_json::to_string::<ReqHtyLabel>(&req_hty_label_create)?;
// //
// //         let resp_create = client
// //             .post(format!("{}/create_or_update_labels", get_uc_url()))
// //             .body(json_params_create)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         let label_new = serde_json::from_str::<HtyResponse<ReqHtyLabel>>(
// //             resp_create.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(
// //             ActionLabel::verify_exist_by_action_id_and_label_id(
// //                 &params["hty_action1_id"].clone(),
// //                 &label_new.hty_label_id.clone().unwrap(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             true,
// //         )?;
// //
// //         let req_hty_label_update = ReqHtyLabel {
// //             hty_label_id: label_new.hty_label_id.clone(),
// //             label_name: Some("test_label_1".to_string()),
// //             label_desc: None,
// //             label_status: Some(APP_STATUS_ACTIVE.to_string()),
// //             roles: None,
// //             actions: None,
// //         };
// //
// //         let json_params_update = serde_json::to_string::<ReqHtyLabel>(&req_hty_label_update)?;
// //
// //         let resp_update = client
// //             .post(format!("{}/create_or_update_labels", get_uc_url()))
// //             .body(json_params_update)
// //             .header(Header::new("HtySudoerToken", token.clone()))
// //             .header(ContentType::JSON)
// //             .dispatch();
// //
// //         serde_json::from_str::<HtyResponse<ReqHtyLabel>>(
// //             resp_update.into_string().clone().unwrap().as_str(),
// //         )
// //             .unwrap()
// //             .d
// //             .unwrap();
// //
// //         my_assert_eq(
// //             ActionLabel::verify_exist_by_action_id_and_label_id(
// //                 &params["hty_action1_id"].clone(),
// //                 &label_new.hty_label_id.clone().unwrap(),
// //                 &conn,
// //             )
// //                 .unwrap(),
// //             false,
// //         )?;
// //
// //         Ok(())
// //     };
// //     do_test(Box::new(my_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_verify_jwt_token() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         {
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //             let mut in_params = params.clone();
// //             in_params.insert("username".to_string(), "test_user3".to_string());
// //             in_params.insert("password".to_string(), "testpass".to_string());
// //             in_params.insert("hty_id".to_string(), in_params["hty_id3"].clone());
// //
// //             verify_user(&in_params, &client, &params["root_token"].clone())?;
// //
// //             let req_login = ReqLogin {
// //                 username: Some(in_params["username"].clone()),
// //                 password: Some(in_params["password"].clone()),
// //             };
// //
// //             let json_params = serde_json::to_string::<ReqLogin>(&req_login)?;
// //
// //             let resp = client
// //                 .post(format!("{}/login_with_password", get_uc_url()))
// //                 .body(json_params)
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //             let resp_text = resp.into_string();
// //
// //             let token = jwt_decode_token(serde_json::from_str::<HtyResponse<String>>(resp_text.unwrap().as_str()).unwrap().d.unwrap());
// //
// //             let token_jwt = jwt_encode_token(token.unwrap())?;
// //
// //             //Test verify jwt token
// //             client
// //                 .post(format!("{}/verify_jwt_token", get_uc_url()))
// //                 .header(Header::new("Authorization", token_jwt.clone()))
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_login_with_cert() {
// //     let web_test = |_params: HashMap<String, String>| -> anyhow::Result<()>
// //         {
// //             let conn = &get_conn(&db::pool(&get_uc_db_url()));
// //             let app = HtyApp::find_by_domain("mocked_app", conn)?;
// //
// //             debug!("Test hty app id is: {}", app.clone().app_id);
// //
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //
// //             let priv_key = app.privkey.unwrap();
// //             let pub_key = app.pubkey.unwrap();
// //
// //             let encrypt_text = encrypt_text_with_private_key(priv_key.clone(), pub_key.clone())?;
// //
// //             info!("priv_key -> {} / pub_key -> {} / encrypt_text -> {}", priv_key, pub_key, encrypt_text);
// //
// //             let req_cert = ReqCert {
// //                 encrypted_data: Some(encrypt_text.clone()),
// //             };
// //
// //             let json_params = serde_json::to_string::<ReqCert>(&req_cert)?;
// //             let resp = client
// //                 .post(format!("{}/login_with_cert", get_uc_url()))
// //                 .body(json_params)
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .dispatch();
// //
// //             assert_eq!(200, resp.status().code);
// //
// //             let resp_text = resp.into_string();
// //             debug!("Login with cert response {}", resp_text.unwrap());
// //
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // #[test]
// // fn test_generate_key_pair() {
// //     let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //         {
// //             let _conn = &get_conn(&db::pool(&get_uc_db_url()));
// //
// //             let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //
// //             let token = params["root_token"].clone();
// //
// //             let resp = client
// //                 .get(format!("{}/generate_key_pair", get_uc_url()))
// //                 .header(ContentType::JSON)
// //                 .header(Header::new("HtyHost", "mocked_app"))
// //                 .header(Header::new("HtySudoerToken", token.clone()))
// //                 .dispatch();
// //             let resp_text = resp.into_string();
// //
// //             //debug!("Generate key pair response {}", resp_text.unwrap());
// //
// //             let key_pair = serde_json::from_str::<HtyResponse<HtyKeyPair>>(
// //                 resp_text.clone().unwrap().as_str(),
// //             )
// //                 .unwrap()
// //                 .d
// //                 .unwrap();
// //
// //             debug!("Request get generate key pair is {:?}" , key_pair);
// //
// //             Ok(())
// //         };
// //
// //     do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
// //
// // // leak API, already disabled
// // #[ignore]
// // #[test]
// // fn test_get_encrypt_id_with_pubkey() {
// //     // let web_test = |params: HashMap<String, String>| -> anyhow::Result<()>
// //     //     {
// //     //         let client = Client::untracked(uc_rocket(random_port(), &get_uc_db_url()))?;
// //     //         let mut in_params = params.clone();
// //     //         in_params.insert("pubkey".to_string(), get_hty_app_pubkey());
// //     //
// //     //         let req_pubkey= ReqPubkey {
// //     //             pubkey: Some(in_params["pubkey"].clone()),
// //     //         };
// //     //
// //     //         let json_params = serde_json::to_string::<ReqPubkey>(&req_pubkey)?;
// //     //         let resp = client
// //     //             .post(format!("{}/get_encrypt_id_with_pubkey", get_uc_url()))
// //     //             .body(json_params)
// //     //             .header(ContentType::JSON)
// //     //             .header(Header::new("HtyHost", "localhost"))
// //     //             .dispatch();
// //     //         let resp_text = resp.into_string();
// //     //         let encrypt_app_id = serde_json::from_str::<HtyResponse<String>>(resp_text.unwrap().as_str()).unwrap().d.unwrap();
// //     //         debug!("Get encrypt app id {:?} with pubkey {:?}" , encrypt_app_id.clone(), get_hty_app_pubkey());
// //     //
// //     //         Ok(())
// //     //     };
// //     //
// //     // do_test(Box::new(web_test), Rc::new(Box::new(HtyucTestScaffold {})));
// // }
//
