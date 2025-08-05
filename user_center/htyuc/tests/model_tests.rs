// use htycommons::{db, uuid};
// use htyuc::models::{HtyApp, HtyResource, HtyUser, UserAppInfo};
// use std::collections::HashMap;
//
//
// use htycommons::db::{get_conn, get_uc_db_url};
// use htycommons::test_scaffold::{do_test, my_assert_eq};
// use std::rc::Rc;
// use htycommons::logger::{info};
// use htyuc::test_scaffold::HtyucTestScaffold;
//
//
// #[test]
// #[warn(unused_must_use)]
// pub fn basic_test() {
//     let f = move |_: HashMap<String, String>| -> anyhow::Result<()> {
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_find_all_users_by_domain() {
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = get_conn(&db::pool(&get_uc_db_url()));
//         let users = HtyUser::all_users_by_app_domain("mocked_app", &conn);
//         let in_params = params.clone();
//
//         let mut hty_ids = vec![in_params["hty_id1"].clone(), in_params["hty_id2"].clone()];
//         let mut user_ids = users
//             .unwrap()
//             .iter()
//             .map(|u| u.hty_id.clone())
//             .collect::<Vec<String>>();
//
//         my_assert_eq(hty_ids.sort(), user_ids.sort())?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_create_user_with_info() {
//     let info = UserAppInfo {
//         hty_id: "".to_string(),
//         app_id: Some("".to_string()),
//         openid: Some("".to_string()),
//         is_registered: false,
//         id: uuid(),
//         username: None,
//         password: None,
//         meta: None,
//         created_at: None,
//         teacher_info: None,
//         student_info: None,
//     };
//
//     let user = HtyUser {
//         hty_id: uuid(),
//         enabled: false,
//         created_at: None,
//         union_id: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     };
//
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let mut cloned_info = info.clone();
//         let in_params = params.clone();
//         cloned_info.app_id = Some(in_params["app_id"].to_string());
//         let hty_id = HtyUser::create_with_info(&user, &Some(cloned_info), conn)?;
//         UserAppInfo::delete_all_by_hty_id(hty_id.as_str(), conn)?;
//         HtyUser::delete_by_id(hty_id.as_str(), conn)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_find_by_username_and_app_id() {
//     let info = UserAppInfo {
//         hty_id: "".to_string(),
//         app_id: Some("".to_string()),
//         openid: Some("".to_string()),
//         is_registered: false,
//         id: uuid(),
//         username: Some("moicenmoicen".to_string()),
//         password: None,
//         meta: None,
//         created_at: None,
//         teacher_info: None,
//         student_info: None,
//     };
//
//     let user = HtyUser {
//         hty_id: uuid(),
//         enabled: false,
//         created_at: None,
//         union_id: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     };
//
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let mut cloned_info = info.clone();
//         let in_params = params.clone();
//         cloned_info.app_id = Some(in_params["app_id"].to_string());
//         let hty_id = HtyUser::create_with_info(&user, &Some(cloned_info), conn)?;
//
//         let find_info = UserAppInfo::find_by_username_and_app_id(&info.clone().username.unwrap().clone(), &in_params["app_id"], conn)?;
//         my_assert_eq(find_info.hty_id, user.hty_id.clone())?;
//
//         UserAppInfo::delete_all_by_hty_id(hty_id.as_str(), conn)?;
//         HtyUser::delete_by_id(hty_id.as_str(), conn)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_update_user_with_info() {
//     let user_app_info = UserAppInfo {
//         hty_id: "".to_string(),
//         app_id: Some("".to_string()),
//         openid: Some("".to_string()),
//         is_registered: false,
//         id: uuid(),
//         username: None,
//         password: None,
//         meta: None,
//         created_at: None,
//         teacher_info: None,
//         student_info: None,
//     };
//
//     let user = HtyUser {
//         hty_id: uuid(),
//         union_id: None,
//         enabled: false,
//         created_at: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     };
//
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let in_params = params.clone();
//
//         let mut cloned_info = user_app_info.clone();
//         cloned_info.app_id = Some(in_params["app_id"].to_string());
//
//         let hty_id = HtyUser::create_with_info(&user, &Some(cloned_info), conn).unwrap();
//
//         let u_user = HtyUser::find_by_id(hty_id.as_str(), conn)?;
//
//         let u_info = UserAppInfo::find_all_by_hty_id(hty_id.as_str(), conn)?;
//
//         HtyUser::update_user_with_info(&Some(u_user), &Some(u_info[0].clone()), conn)?;
//         UserAppInfo::delete_all_by_hty_id(hty_id.as_str(), conn)?;
//         HtyUser::delete_by_id(hty_id.as_str(), conn)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// #[warn(unused_must_use)]
// pub fn join_test() {
//     let f = move |_: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let vec = HtyUser::first_with_info(conn);
//         let (wx_info, hty_user) = vec.unwrap().to_vec().get(0).unwrap().clone();
//         my_assert_eq(wx_info.hty_id, hty_user.hty_id)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// #[warn(unused_must_use)]
// pub fn test_wx_app() {}
//
// #[test]
// pub fn test_find_by_domain() {
//     let f = move |_: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let app = HtyApp::find_by_domain(&"mocked_app".to_string(), conn);
//         my_assert_eq(Some("mocked_app".to_string()), app.unwrap().domain)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_create_hty_resource() {
//     let f = move |_: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let resource = HtyResource {
//             filename: None,
//             app_id: String::from("test_app_id"),
//             hty_resource_id: String::from("test_resource_1"),
//             created_at: None,
//             url: String::from("test_url"),
//             res_type: None,
//             created_by: None,
//             task_id: None,
//         };
//         let test_resource = HtyResource::create(&resource, conn);
//
//         info(format!("---- {:?}", test_resource).as_str());
//         my_assert_eq(
//             "test_resource_1",
//             test_resource.unwrap().hty_resource_id.as_str(),
//         )?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_find_hty_resources_by_task_id() {
//     let f = move |_: HashMap<String, String>| -> anyhow::Result<()> {
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let resource = HtyResource {
//             filename: None,
//             app_id: String::from("test_app_id"),
//             hty_resource_id: String::from("test_resource_1"),
//             created_at: None,
//             url: String::from("test_url"),
//             res_type: None,
//             created_by: None,
//             task_id: Some(String::from("test_task_id")),
//         };
//         let _test_resource = HtyResource::create(&resource, conn);
//         let find_resource = HtyResource::find_all_by_task_id("test_task_id", conn);
//
//         my_assert_eq(_test_resource.unwrap().task_id, find_resource.unwrap()[0].clone().task_id)?;
//
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_roles_by_app_id() {
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()>{
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let user_info = UserAppInfo::find_by_hty_id_and_app_id(&params["hty_id1"], &"test_app_id".to_string(), conn)?;
//         let res = user_info.roles_by_id(conn)?;
//         my_assert_eq(res.clone().unwrap().len(), 3)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }
//
// #[test]
// pub fn test_req_roles_by_id() {
//     let f = move |params: HashMap<String, String>| -> anyhow::Result<()>{
//         let conn = &get_conn(&db::pool(&get_uc_db_url()));
//         let user_info = UserAppInfo::find_by_hty_id_and_app_id(&params["hty_id1"], &"test_app_id".to_string(), conn)?;
//         let res = user_info.req_roles_by_id(conn)?.unwrap();
//         my_assert_eq(res.clone().len(), 3)?;
//         let req_role = res[0].clone();
//         my_assert_eq(req_role.clone().labels.unwrap().clone().len(), 1)?;
//         my_assert_eq(req_role.clone().actions.unwrap().clone().len(), 1)?;
//         Ok(())
//     };
//
//     do_test(Box::new(f), Rc::new(Box::new(HtyucTestScaffold {})));
// }