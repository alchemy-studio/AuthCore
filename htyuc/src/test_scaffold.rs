// use htycommons::db::get_uc_db_url;
// use htycommons::logger::logger_init;
// use htycommons::common::{APP_STATUS_ACTIVE, current_local_datetime};
// use htycommons::{db, pass_or_panic, uuid};
// use htycommons::cert::{generate_cert_key_pair};
// use std::collections::HashMap;
// // use std::ops::DerefMut;
// use std::sync::Once;
// use diesel::PgConnection;
// use htycommons::jwt::jwt_encode_token;
// use htycommons::test_scaffold::TestScaffold;
// use htycommons::web::{HtyToken, ReqHtyLabel, ReqHtyRole};
// use crate::{ActionLabel, AppFromTo, AppRole, HtyAction, HtyApp, HtyGongGao, HtyLabel, HtyResource, HtyRole, HtyTemplate, HtyTemplateData, HtyTongzhi, HtyUser, HtyUserGroup, RoleAction, RoleLabel, UserAppInfo, UserInfoRole};
// use crate::ddl::uc_ddl;
// use crate::models::{HtyTag, HtyTagRef};
//
// static INIT: Once = Once::new();
//
// pub fn test_init() {
//     INIT.call_once(|| {
//         logger_init();
//         ()
//     });
// }
//
// pub fn mocked_user(uuid: &str) -> HtyUser {
//     HtyUser {
//         hty_id: uuid.to_string(),
//         union_id: None,
//         enabled: false,
//         created_at: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     }
// }
//
// pub fn mocked_user_with_unionid(uuid: &str) -> HtyUser {
//     HtyUser {
//         hty_id: uuid.to_string(),
//         union_id: Some("test_union".to_string()),
//         enabled: false,
//         created_at: None,
//         real_name: None,
//         sex: None,
//         mobile: None,
//     }
// }
//
// pub fn mocked_app(app_id: &str, secret: &str, domain: &str) -> HtyApp {
//     let key_pair = generate_cert_key_pair().unwrap();
//     let priv_key = key_pair.privkey;
//     let pub_key = key_pair.pubkey;
//
//     HtyApp {
//         app_id: app_id.to_string(),
//         wx_secret: Some(secret.to_string()),
//         domain: Some(domain.to_string()),
//         app_status: APP_STATUS_ACTIVE.to_string(),
//         app_desc: None,
//         pubkey: pub_key,
//         privkey: priv_key,
//         wx_id: None,
//         is_wx_app: Some(false),
//     }
// }
//
// pub fn mocked_info(
//     openid: &str,
//     hty_id: &str,
//     app_id: &str,
//     username: &str,
//     password: &str,
//     is_registered: bool,
// ) -> UserAppInfo {
//     UserAppInfo {
//         openid: Some(openid.to_string()),
//         hty_id: hty_id.to_string(),
//         app_id: Some(app_id.to_string()),
//         is_registered,
//         id: uuid(),
//         username: Some(username.to_string()),
//         password: Some(password.to_string()),
//         meta: None,
//         created_at: None,
//         teacher_info: None,
//         student_info: None,
//         reject_reason: None,
//         needs_refresh: None,
//         avatar_url: None,
//     }
// }
//
// pub fn mocked_info_role(id: &str, user_info_id: &str, role_id: &str) -> UserInfoRole {
//     UserInfoRole {
//         the_id: id.to_string(),
//         user_info_id: user_info_id.to_string(),
//         role_id: role_id.to_string(),
//     }
// }
//
// pub fn mocked_hty_role(role_id: &str, role_name: &str, role_desc: &str) -> HtyRole {
//     HtyRole {
//         hty_role_id: role_id.to_string(),
//         role_key: role_name.to_string(),
//         role_desc: Some(role_desc.to_string()),
//         role_status: APP_STATUS_ACTIVE.to_string(),
//         style: None,
//         role_name: None,
//     }
// }
//
// pub fn mocked_hty_label(label_id: &str, label_name: &str, label_desc: &str) -> HtyLabel {
//     HtyLabel {
//         hty_label_id: label_id.to_string(),
//         label_name: label_name.to_string(),
//         label_desc: Some(label_desc.to_string()),
//         label_status: APP_STATUS_ACTIVE.to_string(),
//         style: None,
//     }
// }
//
// pub fn mocked_hty_action(action_id: &str, action_name: &str, action_desc: &str) -> HtyAction {
//     HtyAction {
//         hty_action_id: action_id.to_string(),
//         action_name: action_name.to_string(),
//         action_desc: Some(action_desc.to_string()),
//         action_status: APP_STATUS_ACTIVE.to_string(),
//     }
// }
//
// pub fn mocked_role_label(the_id: &str, role_id: &str, label_id: &str) -> RoleLabel {
//     RoleLabel {
//         the_id: the_id.to_string(),
//         role_id: role_id.to_string(),
//         label_id: label_id.to_string(),
//     }
// }
//
// pub fn mocked_role_action(the_id: &str, role_id: &str, action_id: &str) -> RoleAction {
//     RoleAction {
//         the_id: the_id.to_string(),
//         role_id: role_id.to_string(),
//         action_id: action_id.to_string(),
//     }
// }
//
// pub fn mocked_action_label(the_id: &str, action_id: &str, label_id: &str) -> ActionLabel {
//     ActionLabel {
//         the_id: the_id.to_string(),
//         action_id: action_id.to_string(),
//         label_id: label_id.to_string(),
//     }
// }
//
// pub fn mocked_app_role(the_id: &str, app_id: &str, role_id: &str) -> AppRole {
//     AppRole {
//         the_id: the_id.to_string(),
//         app_id: app_id.to_string(),
//         role_id: role_id.to_string(),
//     }
// }
//
// pub fn get_mocked_root_token() -> anyhow::Result<String> {
//     let label_root = ReqHtyLabel {
//         hty_label_id: None,
//         label_name: Some("SYS_ROOT".to_string()),
//         label_desc: None,
//         label_status: None,
//         roles: None,
//         actions: None,
//         style: None,
//     };
//
//     let role_root = ReqHtyRole {
//         hty_role_id: None,
//         user_app_info_id: None,
//         app_ids: None,
//         role_key: None,
//         role_desc: None,
//         role_status: None,
//         labels: Some(vec![label_root]),
//         actions: None,
//         style: None,
//         role_name: None,
//     };
//
//     let token = HtyToken {
//         token_id: uuid(),
//         hty_id: Some(uuid()),
//         app_id: None,
//         ts: current_local_datetime(),
//         roles: Some(vec![role_root]),
//         tags: None,
//     };
//
//     Ok(jwt_encode_token(token)?)
// }
//
// pub struct HtyucTestScaffold {}
//
// impl TestScaffold for HtyucTestScaffold {
//     fn before_test(self: &Self) -> anyhow::Result<HashMap<String, String>> {
//         dotenv::dotenv().ok();
//         let uc_pool = db::pool(&get_uc_db_url());
//         // let conn = *&uc_pool.get().unwrap().deref_mut();
//         test_init();
//         delete_all_uc(&mut uc_pool.get().unwrap());
//         uc_ddl();
//
//         let hty_id1 = uuid();
//
//         let new_user1 = mocked_user(hty_id1.as_str());
//
//         let _created_user1 = HtyUser::create(&new_user1, &mut uc_pool.get().unwrap());
//
//         // ----
//         let hty_id2 = uuid();
//         let new_user2 = mocked_user(hty_id2.as_str());
//
//         let _created_user2 = HtyUser::create(&new_user2, &mut uc_pool.get().unwrap());
//
//
//         // ----
//         let hty_id3 = uuid();
//         let new_user3 = mocked_user_with_unionid(hty_id3.as_str());
//         let _created_user3 = HtyUser::create(&new_user3, &mut uc_pool.get().unwrap());
//
//         let app_id = "test_app_id";
//
//         let new_app = mocked_app(app_id, "test_secret", "mocked_app");
//
//         let _created_app = HtyApp::create(&new_app, &mut uc_pool.get().unwrap());
//
//         // ---
//
//         let new_info1 = mocked_info(
//             "test_openid1",
//             hty_id1.as_str(),
//             app_id,
//             "test_user1",
//             "",
//             false,
//         );
//
//         let created_info1 = UserAppInfo::create(&new_info1.clone(), &mut uc_pool.get().unwrap())?;
//
//         let hty_role1 = mocked_hty_role(uuid().as_str(), "test_read_role", "");
//         let created_hty_role1 = HtyRole::create(&hty_role1, &mut uc_pool.get().unwrap())?;
//
//         let app_role = mocked_app_role(
//             uuid().as_str(),
//             app_id.clone(),
//             hty_role1.clone().hty_role_id.as_str(),
//         );
//
//         AppRole::create(&app_role, &mut uc_pool.get().unwrap())?;
//
//         let hty_action1 = mocked_hty_action(uuid().as_str(), "test_read_action", "");
//         let _created_hty_action1 = HtyAction::create(&hty_action1, &mut uc_pool.get().unwrap());
//
//         let hty_label1 = mocked_hty_label(uuid().as_str(), "test_read_label", "");
//         let _created_hty_label1 = HtyLabel::create(&hty_label1, &mut uc_pool.get().unwrap());
//
//         let hty_role2 = mocked_hty_role(uuid().as_str(), "test_write_role", "");
//         let created_hty_role2 = HtyRole::create(&hty_role2, &mut uc_pool.get().unwrap())?;
//
//         let app_role2 = mocked_app_role(
//             uuid().as_str(),
//             app_id.clone(),
//             hty_role2.clone().hty_role_id.as_str(),
//         );
//
//         AppRole::create(&app_role2, &mut uc_pool.get().unwrap())?;
//
//         let hty_action2 = mocked_hty_action(uuid().as_str(), "test_write_action", "");
//         let _created_hty_action2 = HtyAction::create(&hty_action2, &mut uc_pool.get().unwrap());
//
//         let hty_label2 = mocked_hty_label(uuid().as_str(), "test_write_label", "");
//         let _created_hty_label2 = HtyLabel::create(&hty_label2, &mut uc_pool.get().unwrap());
//
//         let hty_role3 = mocked_hty_role(uuid().as_str(), "test_execute_role", "");
//         let created_hty_role3 = HtyRole::create(&hty_role3, &mut uc_pool.get().unwrap())?;
//
//         let hty_action3 = mocked_hty_action(uuid().as_str(), "test_execute_action", "");
//         let _created_hty_action3 = HtyAction::create(&hty_action3, &mut uc_pool.get().unwrap());
//
//         let hty_label3 = mocked_hty_label(uuid().as_str(), "test_execute_label", "");
//         let _created_hty_label3 = HtyLabel::create(&hty_label3, &mut uc_pool.get().unwrap());
//
//         let user1_info_role1 = mocked_info_role(
//             uuid().as_str(),
//             created_info1.clone().id.clone().as_str(),
//             created_hty_role1.clone().hty_role_id.clone().as_str(),
//         );
//         let _c_user1_info_role1 = UserInfoRole::create(&user1_info_role1, &mut uc_pool.get().unwrap());
//
//         let user1_info_role2 = mocked_info_role(
//             uuid().as_str(),
//             created_info1.clone().id.clone().as_str(),
//             created_hty_role2.clone().hty_role_id.clone().as_str(),
//         );
//         let _c_user1_info_role2 = UserInfoRole::create(&user1_info_role2, &mut uc_pool.get().unwrap());
//
//         let user1_info_role3 = mocked_info_role(
//             uuid().as_str(),
//             created_info1.clone().id.clone().as_str(),
//             created_hty_role3.clone().hty_role_id.clone().as_str(),
//         );
//         let _c_user1_info_role3 = UserInfoRole::create(&user1_info_role3, &mut uc_pool.get().unwrap());
//
//         let role1_action1 = mocked_role_action(
//             uuid().as_str(),
//             hty_role1.clone().hty_role_id.as_str(),
//             hty_action1.clone().hty_action_id.as_str(),
//         );
//         let _c_role1_action1 = RoleAction::create(&role1_action1, &mut uc_pool.get().unwrap());
//
//         let role1_label1 = mocked_role_label(
//             uuid().as_str(),
//             hty_role1.clone().hty_role_id.as_str(),
//             hty_label1.clone().hty_label_id.as_str(),
//         );
//         let _c_role1_action = RoleLabel::create(&role1_label1, &mut uc_pool.get().unwrap());
//
//         let role2_action2 = mocked_role_action(
//             uuid().as_str(),
//             hty_role2.clone().hty_role_id.as_str(),
//             hty_action2.clone().hty_action_id.as_str(),
//         );
//         let _c_role2_action2 = RoleAction::create(&role2_action2, &mut uc_pool.get().unwrap());
//
//         let action1_label1 = mocked_action_label(
//             uuid().as_str(),
//             hty_action1.clone().hty_action_id.as_str(),
//             hty_label1.clone().hty_label_id.as_str(),
//         );
//         let _c_action1_label1 = ActionLabel::create(&action1_label1, &mut uc_pool.get().unwrap());
//
//         // ----
//         //
//         let new_info2 = mocked_info(
//             "test_openid2",
//             hty_id2.as_str(),
//             app_id,
//             "test_user2",
//             "",
//             false,
//         );
//         let created_info2 = UserAppInfo::create(&new_info2, &mut uc_pool.get().unwrap())?;
//
//         let user2_info_role3 = mocked_info_role(
//             uuid().as_str(),
//             created_info2.clone().id.clone().as_str(),
//             created_hty_role3.clone().hty_role_id.clone().as_str(),
//         );
//         let _c_user1_info_role3 = UserInfoRole::create(&user2_info_role3, &mut uc_pool.get().unwrap());
//
//         let new_info3 = mocked_info(
//             "test_openid3",
//             hty_id3.as_str(),
//             app_id,
//             "test_user3",
//             "testpass",
//             false,
//         );
//         let created_info3 = UserAppInfo::create(&new_info3, &mut uc_pool.get().unwrap())?;
//
//         let mut params = HashMap::new();
//         params.insert("hty_id1".to_string(), hty_id1);
//         params.insert("created_info1".to_string(), created_info1.clone().id);
//         params.insert("hty_id2".to_string(), hty_id2);
//         params.insert("created_info2".to_string(), created_info2.clone().id);
//         params.insert("hty_id3".to_string(), hty_id3);
//         params.insert("created_info3".to_string(), created_info3.clone().id);
//         params.insert("app_id".to_string(), app_id.to_string());
//         params.insert("hty_role1_id".to_string(), hty_role1.clone().hty_role_id);
//         params.insert("hty_role2_id".to_string(), hty_role2.clone().hty_role_id);
//         params.insert("hty_role3_id".to_string(), hty_role3.clone().hty_role_id);
//         params.insert(
//             "hty_action1_id".to_string(),
//             hty_action1.clone().hty_action_id,
//         );
//         params.insert(
//             "hty_action2_id".to_string(),
//             hty_action2.clone().hty_action_id,
//         );
//         params.insert(
//             "hty_action3_id".to_string(),
//             hty_action3.clone().hty_action_id,
//         );
//         params.insert("hty_label1_id".to_string(), hty_label1.clone().hty_label_id);
//         params.insert("hty_label2_id".to_string(), hty_label2.clone().hty_label_id);
//         params.insert("hty_label3_id".to_string(), hty_label3.clone().hty_label_id);
//
//         let token = get_mocked_root_token()?;
//         params.insert("root_token".to_string(), token);
//
//         Ok(params)
//     }
//
//     fn after_test(self: &Self) {
//         // let _ = dotenv::dotenv();
//         // let uc_pool = db::pool(&get_uc_db_url());
//         // match uc_pool.get() {
//         //     Ok(conn) => {
//         //         delete_all_uc(conn);
//         //     }
//         //     Err(e) => {
//         //         panic!("[AFTER TEST ERROR] -> {:?}", e)
//         //     }
//         // }
//     }
// }
//
// pub fn delete_all_uc(conn: &mut PgConnection) {
//     // this should be deleted firstly
//     pass_or_panic(HtyUserGroup::delete_all(conn));
//
//     pass_or_panic(HtyResource::delete_all(conn));
//
//     pass_or_panic(HtyTongzhi::delete_all(conn));
//
//     pass_or_panic(ActionLabel::delete_all(conn));
//
//     pass_or_panic(AppRole::delete_all(conn));
//
//     pass_or_panic(RoleAction::delete_all(conn));
//
//     pass_or_panic(RoleLabel::delete_all(conn));
//
//     pass_or_panic(HtyAction::delete_all(conn));
//
//     pass_or_panic(HtyLabel::delete_all(conn));
//
//     pass_or_panic(HtyTagRef::delete_all(conn));
//
//     pass_or_panic(HtyTag::delete_all(conn));
//
//     pass_or_panic(UserInfoRole::delete_all(conn));
//
//     pass_or_panic(HtyRole::delete_all(conn));
//
//     // delete all left infos
//     pass_or_panic(UserAppInfo::delete_all(conn));
//
//     // delete all left users
//     pass_or_panic(HtyUser::delete_all(conn));
//
//     pass_or_panic(AppFromTo::delete_all(conn));
//
//     pass_or_panic(HtyGongGao::delete_all(conn));
//
//     pass_or_panic(HtyTemplateData::<String>::delete_all(conn));
//
//     pass_or_panic(HtyTemplate::delete_all(conn));
//
//     // delete all apps
//     pass_or_panic(HtyApp::delete_all(conn));
// }
