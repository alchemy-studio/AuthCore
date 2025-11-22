use std::ops::DerefMut;
use std::sync::Arc;
use anyhow::anyhow;
use diesel::PgConnection;
use htycommons::common::{current_local_datetime, HtyErr, HtyErrCode};
// use htycommons::db;
use htycommons::web::{get_music_room_mini_url, HtyHostHeader};
use htycommons::wx::{ReqWxMessageData2keywordTemplate, ReqWxMessageData3KeywordTemplate, ReqWxMessageDataValue, ReqWxMiniProgram, ReqWxPushMessage};
use serde::Serialize;
use chrono::Datelike;
use tracing::debug;
use htycommons::db::{DbState, extract_conn, fetch_db_conn, UNREAD};
use htycommons::models::PushInfo;
use htyuc_models::send_tongzhi;
use crate::{AppFromTo, CommonTongzhiContent, HtyApp, HtyRole, HtyTemplateData, HtyTongzhi, HtyUser, push_wx_message, TongzhiMeta, UserAppInfo};

#[allow(dead_code)]
fn build_wx_push_message_for_teacher_register(
    type_notify: &String,
    real_name: &String,
    from_app: &HtyApp,
    to_app: &HtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<Vec<ReqWxPushMessage<ReqWxMessageData2keywordTemplate>>> {
    debug!("build_wx_push_message_for_teacher_register -> real_name: {:?} / from_app: {:?} / to_app: {:?}", real_name, from_app, to_app);

    // let in_role_id = id_role.clone();

    // todo: just return ADMINs of this APP
    let admin_users =
        HtyRole::find_by_key(&String::from("ADMIN"), conn)?.find_linked_user_app_info(conn)?;

    debug!(
        "build_wx_push_message_for_teacher_register -> admins: {:?}",
        &admin_users
    );

    let from_app_admin_user_infos: Vec<UserAppInfo> = admin_users
        .into_iter()
        .filter(|the_info| the_info.app_id.as_ref().map(|id| id == &from_app.app_id).unwrap_or(false))
        .collect();

    debug!("from_app_admin_users -> {:?}", &from_app_admin_user_infos);

    // let template_id = String::from(env::var("WX_MSG_TEACHER_REGISTER")?.to_string());

    let wx_id = from_app.clone().wx_id
        .ok_or_else(|| anyhow::anyhow!("wx_id is required for from_app"))?;
    let wx_mini_program = ReqWxMiniProgram {
        appid: wx_id,
        pagepath: "index".to_string(),
    };

    let current_datetime = current_local_datetime();

    let (in_template, in_template_data) =
        HtyTemplateData::<ReqWxMessageData2keywordTemplate>::find_with_template_key_and_app_id(
            &type_notify,
            &to_app.app_id.clone(),
            conn)?;


    debug!("build_wx_push_message_for_teacher_register -> {:?} / {:?}", &in_template, &in_template_data);

    let mut wx_message_data = in_template_data.template_text.clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;

    let val_keyword1 = wx_message_data.keyword1.value;
    let val_keyword2 = wx_message_data.keyword2.value;

    wx_message_data.keyword1 = ReqWxMessageDataValue {
        value: val_keyword1.replace("TEACHER_NAME", real_name)
    };

    wx_message_data.keyword2 = ReqWxMessageDataValue {
        value: val_keyword2.replace("YEAR",
                                    current_datetime.year().to_string().as_str())
            .replace("MONTH",
                     current_datetime.month().to_string().as_str())
            .replace("DAY",
                     current_datetime.day().to_string().as_str())
    };

    debug!("build_wx_push_message_for_teacher_register AFTER -> msg: {:?}", &wx_message_data);

    let mut resp = vec![];
    // let mut tongzhi_array = vec![];

    for admin_user_info in from_app_admin_user_infos {
        let admin_user_openid = admin_user_info.openid.clone()
            .ok_or_else(|| anyhow::anyhow!("admin_user openid is required"))?;

        let admin_user = UserAppInfo::find_hty_user_by_openid(&admin_user_openid, conn)?;

        let admin_user_toapp_info =
            UserAppInfo::find_by_hty_id_and_app_id(&admin_user.hty_id, &to_app.app_id, conn)?;

        let template_id = in_template_data.template_val.clone()
            .ok_or_else(|| anyhow::anyhow!("template_val is required"))?;
        let push_message = ReqWxPushMessage {
            touser: admin_user_toapp_info.openid.clone(),
            // touser_hty_id: Some(admin_user.clone().hty_id),
            template_id,
            url: Some(get_music_room_mini_url()), // todo: use PARAM to load domain
            miniprogram: Some(wx_mini_program.clone()),
            data: wx_message_data.clone(),
        };

        resp.push(push_message);
    }

    debug!(
        "build_wx_push_message_for_teacher_register -> resp: {:?}",
        & resp
        );

    Ok(resp)
}

fn build_wx_push_message_for_student_register(
    type_notify: &String,
    _real_name_teacher: &String,
    id_teacher: &String,
    real_name_student: &String,
    _id_student: &String,
    from_app: &HtyApp,
    to_app: &HtyApp,
    _to_role_id: &Option<String>,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqWxPushMessage<ReqWxMessageData2keywordTemplate>> {
    let user_teacher = UserAppInfo::find_by_hty_id_and_app_id(id_teacher, &to_app.app_id, conn)?;
    // let template_id = String::from(env::var("WX_MSG_STUDENT_REGISTER")?.to_string());

    debug!("build_wx_push_message_for_student_register -> user_teacher -> {:?}", user_teacher);

    let wx_id = from_app.clone().wx_id
        .ok_or_else(|| anyhow::anyhow!("wx_id is required for from_app"))?;
    let wx_mini_program = ReqWxMiniProgram {
        appid: wx_id,
        pagepath: "index".to_string(),
    };

    let current_datetime = current_local_datetime();

    let (_in_template, in_template_data) =
        HtyTemplateData::<ReqWxMessageData2keywordTemplate>::find_with_template_key_and_app_id(
            &type_notify.to_string(),
            &to_app.app_id.clone(),
            conn)?;

    debug!("build_wx_push_message_for_student_register -> _in_template -> {:?}", _in_template);
    debug!("build_wx_push_message_for_student_register -> in_template_data -> {:?}", in_template_data);

    let mut wx_message_data = in_template_data.template_text.clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;

    debug!("build_wx_push_message_for_student_register -> BEFORE wx_message_data -> {:?}", wx_message_data);

    let val_keyword1 = wx_message_data.keyword1.value;
    let val_keyword2 = wx_message_data.keyword2.value;

    wx_message_data.keyword1 = ReqWxMessageDataValue {
        value: val_keyword1.replace("STUDENT_NAME", real_name_student)
    };

    wx_message_data.keyword2 = ReqWxMessageDataValue {
        value: val_keyword2.replace("YEAR", current_datetime.year().to_string().as_str())
            .replace("MONTH", current_datetime.month().to_string().as_str())
            .replace("DAY", current_datetime.day().to_string().as_str())
    };

    debug!("build_wx_push_message_for_student_register -> AFTER wx_message_data -> {:?}", wx_message_data);

    let template_id = in_template_data.template_val.clone()
        .ok_or_else(|| anyhow::anyhow!("template_val is required"))?;
    Ok(ReqWxPushMessage {
        touser: user_teacher.openid.clone(),
        template_id,
        url: Some(get_music_room_mini_url()),
        miniprogram: Some(wx_mini_program.clone()),
        data: wx_message_data.clone(),
    })
}

fn build_wx_push_message_for_student_register_success(
    type_notify: &String,
    id_student: &String,
    real_name_teacher: &String,
    from_app: &HtyApp,
    to_app: &HtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqWxPushMessage<ReqWxMessageData2keywordTemplate>> {
    let user_student = UserAppInfo::find_by_hty_id_and_app_id(id_student, &to_app.app_id, conn)?;

    let wx_id = from_app.clone().wx_id
        .ok_or_else(|| anyhow::anyhow!("wx_id is required for from_app"))?;
    let wx_mini_program = ReqWxMiniProgram {
        appid: wx_id,
        pagepath: "index".to_string(),
    };

    let current_datetime = current_local_datetime();

    let (_in_template, in_template_data) =
        HtyTemplateData::<ReqWxMessageData2keywordTemplate>::find_with_template_key_and_app_id(&type_notify,
                                                                                               &to_app.app_id.clone(),
                                                                                               conn)?;

    let mut wx_message_data = in_template_data.template_text.clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;

    let val_keyword1 = wx_message_data.keyword1.value;
    let val_keyword2 = wx_message_data.keyword2.value;

    wx_message_data.keyword1 = ReqWxMessageDataValue {
        value: val_keyword1.replace("TEACHER_NAME", real_name_teacher)
    };

    wx_message_data.keyword2 = ReqWxMessageDataValue {
        value: val_keyword2.replace("YEAR", current_datetime.year().to_string().as_str())
            .replace("MONTH", current_datetime.month().to_string().as_str())
            .replace("DAY", current_datetime.day().to_string().as_str())
    };

    let template_id = in_template_data.template_val.clone()
        .ok_or_else(|| anyhow::anyhow!("template_val is required"))?;
    Ok(ReqWxPushMessage {
        touser: user_student.openid.clone(),
        template_id,
        url: Some(get_music_room_mini_url()),
        miniprogram: Some(wx_mini_program.clone()),
        data: wx_message_data.clone(),
    })
}

fn build_wx_push_message_for_teacher_register_success(
    type_notify: &String,
    id_teacher: &String,
    from_app: &HtyApp,
    to_app: &HtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqWxPushMessage<ReqWxMessageData2keywordTemplate>> {
    debug!("build_wx_push_message_for_teacher_register_success -> id_teacher: {:?} / from_app: {:?} / to_app: {:?} /", id_teacher, from_app, to_app);
    let teacher_toapp_info =
        UserAppInfo::find_by_hty_id_and_app_id(id_teacher, &to_app.app_id, conn)?;
    let teacher_user = HtyUser::find_by_hty_id(id_teacher, conn)?;

    // let template_id = String::from(env::var("WX_MSG_TEACHER_REGISTER_SUCCESS")?.to_string());

    let wx_id = from_app.clone().wx_id
        .ok_or_else(|| anyhow::anyhow!("wx_id is required for from_app"))?;
    let wx_mini_program = ReqWxMiniProgram {
        appid: wx_id,
        pagepath: "index".to_string(),
    };


    let (in_template, in_template_data) =
        HtyTemplateData::<ReqWxMessageData2keywordTemplate>::find_with_template_key_and_app_id(
            &type_notify.to_string(),
            &to_app.app_id.clone(),
            conn)?;

    debug!("build_wx_push_message_for_teacher_register_success -> {:?} / {:?}", &in_template, &in_template_data);

    let mut wx_message_data = in_template_data.template_text.clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;
    let val_keyword1 = wx_message_data.keyword1.value;
    let val_keyword2 = wx_message_data.keyword2.value;

    wx_message_data.keyword1 = ReqWxMessageDataValue {
        value: val_keyword1.replace("TEACHER_NAME", teacher_user.real_name.unwrap_or(String::from("")).as_str())
    };

    wx_message_data.keyword2 = ReqWxMessageDataValue {
        value: val_keyword2.replace("TEACHER_MOBILE", teacher_user.mobile.unwrap_or(String::from("")).as_str())
    };


    let template_id = in_template_data.template_val.clone()
        .ok_or_else(|| anyhow::anyhow!("template_val is required"))?;
    let resp = ReqWxPushMessage {
        touser: teacher_toapp_info.openid,
        template_id,
        url: Some(get_music_room_mini_url()),
        miniprogram: Some(wx_mini_program.clone()),
        data: wx_message_data.clone(),
    };

    debug!(
        "build_wx_push_message_for_teacher_register_success -> RESP: {:?}",
        &resp
    );
    Ok(resp)
}

fn build_wx_push_message_for_reject_register(
    type_notify: &String,
    id_user: &String,
    reject_reason: String,
    from_app: &HtyApp,
    to_app: &HtyApp,
    conn: &mut PgConnection,
) -> anyhow::Result<ReqWxPushMessage<ReqWxMessageData3KeywordTemplate>> {
    debug!("build_wx_push_message_for_reject_register -> id_user: {:?} / reject_reason: {:?} / from_app: {:?} / to_app: {:?}", id_user, reject_reason, from_app, to_app);

    let user = UserAppInfo::find_by_hty_id_and_app_id(id_user, &to_app.app_id, conn)?;

    // let template_id = String::from(env::var("WX_MSG_REJECT_REGISTER")?.to_string());

    let wx_id = from_app.clone().wx_id
        .ok_or_else(|| anyhow::anyhow!("wx_id is required for from_app"))?;
    let wx_mini_program = ReqWxMiniProgram {
        appid: wx_id,
        pagepath: "index".to_string(),
    };

    let current_datetime = current_local_datetime();


    let (in_template, in_template_data) =
        HtyTemplateData::<ReqWxMessageData3KeywordTemplate>::find_with_template_key_and_app_id(
            &type_notify,
            &to_app.app_id.clone(),
            conn)?;

    debug!("build_wx_push_message_for_reject_register -> {:?} / {:?}", &in_template, &in_template_data);

    let mut wx_message_data = in_template_data.template_text.clone()
        .ok_or_else(|| anyhow::anyhow!("template_text is required"))?
        .val
        .ok_or_else(|| anyhow::anyhow!("template_text.val is required"))?;
    debug!("build_wx_push_message_for_reject_register BEFORE -> msg: {:?}", &wx_message_data);
    // keyword1目前是模版里面写死的.
    let val_keyword2 = wx_message_data.keyword2.value;

    wx_message_data.keyword2 = ReqWxMessageDataValue {
        value: val_keyword2.replace("YEAR",
                                    current_datetime.year().to_string().as_str())
            .replace("MONTH",
                     current_datetime.month().to_string().as_str())
            .replace("DAY",
                     current_datetime.day().to_string().as_str())
    };

    let val_keyword3 = wx_message_data.keyword3.value;
    wx_message_data.keyword3 = ReqWxMessageDataValue {
        value: val_keyword3.replace("REJECT_REASON", reject_reason.as_str())
    };

    debug!("build_wx_push_message_for_reject_register AFTER -> msg: {:?}", &wx_message_data);

    let template_id = in_template_data.template_val.clone()
        .ok_or_else(|| anyhow::anyhow!("template_val is required"))?;
    Ok(ReqWxPushMessage {
        touser: user.openid.clone(),
        template_id,
        url: Some(get_music_room_mini_url()),
        miniprogram: Some(wx_mini_program.clone()),
        data: wx_message_data.clone(),
    })
}

pub async fn raw_notify(
    push_info: PushInfo,
    host: HtyHostHeader,
    db_pool: Arc<DbState>,
) -> anyhow::Result<()> {
    debug!(
        "raw_notify -> push_info: {:?}, host: {:?}",
        &push_info, &host
    );
    let push_info_copy = push_info.clone();

    let in_role_id = push_info_copy.to_role_id.clone();

    let user1_id = push_info_copy.hty_id.clone().ok_or(HtyErr {
        code: HtyErrCode::NullErr,
        reason: Some("user1_id is none".to_string()),
    })?;

    let user1 = HtyUser::find_by_hty_id(&user1_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let real_name1 = user1.clone().real_name.unwrap_or(String::from(""));

    debug!("raw_notify -> user1: {:?}", &user1);

    // todo: 未来要使用find_by_label()
    let role_teacher = HtyRole::find_by_key("TEACHER", extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let role_student = HtyRole::find_by_key("STUDENT", extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    let role_admin = HtyRole::find_by_key("ADMIN", extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

    let id_role_teacher = role_teacher.hty_role_id.clone();
    let id_role_student = role_student.hty_role_id.clone();
    let id_role_admin = role_admin.hty_role_id.clone();

    let from_app = if let Some(from_app_id) = push_info.from_app_id.clone() {
        HtyApp::find_by_id(&from_app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?
    } else {
        crate::get_app_from_host((*host).clone(), extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?
    };

    let to_apps = AppFromTo::find_all_active_to_apps_by_from_app(&from_app.app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
    for to_app_rel in to_apps {
        let to_app_id = to_app_rel.to_app_id;
        let to_app = HtyApp::find_by_id(&to_app_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

        debug!("raw_notify -> from_app / {:?}", &from_app);
        debug!("raw_notify -> to_app_id / {:?}", &to_app_id);
        debug!("raw_notify -> to_app / {:?}", &to_app);

        let notify_type = push_info_copy.notify_type.clone();

        match notify_type {
            Some(notify_type) => match notify_type.as_str() {
                "teacher_register" => {
                    let push_messages = build_wx_push_message_for_teacher_register(
                        &notify_type,
                        &real_name1,
                        &from_app,
                        &to_app,
                        // &in_role_id,
                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    )?;

                    for message in push_messages.clone() {
                        // let first_value = push_message.data.first.value.clone();

                        let touser = message.touser.clone()
                            .ok_or_else(|| anyhow::anyhow!("touser is required in message"))?;
                        let send_to = HtyUser::find_by_openid(&touser, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

                        let tongzhi = build_hty_tongzhi(
                            &String::from("teacher_register"),
                            &from_app.app_id,
                            None,
                            &send_to,
                            &message.clone(),
                            &message.clone().data.first.value.clone(),
                            &Some(id_role_admin.clone()),
                            &Some(push_info.clone()),
                        )?;

                        // let _ = HtyTongzhi::create(&tongzhi, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
                        // let async_to_app = to_app.clone();
                        // tokio::spawn(async move {
                        //     let _ = push_wx_message(&async_to_app, &message).await;
                        // });
                        // let c_to_app = to_app.clone();
                        send_tongzhi!(tongzhi, to_app, message, db_pool);
                    }
                }
                "teacher_register_success" => {
                    debug!("raw_notify -> teacher_register_success: start");
                    let push_message = build_wx_push_message_for_teacher_register_success(
                        &notify_type, &user1_id, &from_app, &to_app, extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    )?;
                    debug!(
                        "raw_notify -> teacher_register_success: msg {:?}",
                        &push_message
                    );
                    let first_value = push_message.data.first.value.clone();
                    let tongzhi = build_hty_tongzhi(
                        &notify_type,
                        &from_app.app_id,
                        None,
                        &user1,
                        &push_message,
                        &first_value,
                        &Some(id_role_teacher.clone()),
                        &Some(push_info.clone()),
                    )?;

                    // let _ = HtyTongzhi::create(&tongzhi, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
                    // tokio::spawn(async move {
                    //     let _ = push_wx_message(&to_app, &push_message).await;
                    // });
                    // let c_to_app = to_app.clone();
                    send_tongzhi!(tongzhi, to_app, push_message, db_pool);
                }
                "student_register" => {
                    debug!("raw_notify -> student_register: start");
                    let user2_id = push_info_copy.hty_id2.clone().unwrap_or_default();
                    let user2 = HtyUser::find_by_hty_id(&user2_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
                    let real_name2 = user2.clone().real_name.clone().unwrap_or(String::from(""));
                    debug!("raw_notify -> student_register: user2 {:?}", &user2);

                    let push_message = build_wx_push_message_for_student_register(
                        &notify_type,
                        &real_name1,
                        &user1_id,
                        &real_name2,
                        &user2_id,
                        &from_app,
                        &to_app,
                        &in_role_id,
                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    )?;

                    let first_value = push_message.data.first.value.clone();
                    let tongzhi = build_hty_tongzhi(
                        &notify_type,
                        &from_app.app_id,
                        None,
                        &user1,
                        &push_message,
                        &first_value,
                        &Some(id_role_teacher.clone()),
                        &Some(push_info.clone()),
                    )?;
                    // let _ = HtyTongzhi::create(&tongzhi, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;

                    // debug!("raw_notify -> student_register: msg {:?}", &push_message);
                    // tokio::spawn(async move {
                    //     let _ = push_wx_message(&to_app, &push_message).await;
                    // });
                    // let c_to_app = to_app.clone();
                    send_tongzhi!(tongzhi, to_app, push_message, db_pool);
                }
                "student_register_success" => {
                    debug!("raw_notify -> student_register_success: start");

                    let user2_id = push_info_copy.hty_id2.clone().unwrap_or_default();

                    debug!(
                        "raw_notify -> student_register_success: user2_id: {:?}",
                        &user2_id
                    );

                    let user2 = HtyUser::find_by_hty_id(&user2_id, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
                    let real_name2 = user2.clone().real_name.unwrap_or(String::from(""));

                    debug!(
                        "raw_notify -> student_register_success: user2: {:?}",
                        &user2
                    );

                    let push_message = build_wx_push_message_for_student_register_success(
                        &notify_type,
                        &user1_id,
                        &real_name2,
                        &from_app,
                        &to_app,
                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    )?;
                    let first_value = push_message.data.first.value.clone();
                    let tongzhi = build_hty_tongzhi(
                        &notify_type,
                        &from_app.app_id,
                        None,
                        &user1,
                        &push_message,
                        &first_value,
                        &Some(id_role_student.clone()),
                        &Some(push_info.clone()),
                    )?;
                    // let _ = HtyTongzhi::create(&tongzhi, extract_conn(fetch_db_conn(&db_pool)?).deref_mut())?;
                    //
                    // debug!("raw_notify -> build_wx_push_message_for_student_register_success: msg: {:?}", &push_message);
                    //
                    // tokio::spawn(async move {
                    //     let _ = push_wx_message(&to_app, &push_message).await;
                    // });

                    send_tongzhi!(tongzhi, to_app, push_message, db_pool);
                }
                "reject_register" => {
                    let reject_reason = push_info_copy.reject_reason.clone().unwrap_or_default();
                    let push_message = build_wx_push_message_for_reject_register(
                        &notify_type,
                        &user1_id,
                        reject_reason,
                        &from_app,
                        &to_app,
                        extract_conn(fetch_db_conn(&db_pool)?).deref_mut(),
                    )?;

                    debug!("raw_notify -> reject_register: msg: {:?}", &push_message);

                    // let user_send_to = HtyUser::find_by_id(&user1_id, conn)?;
                    // let first_value = push_message.data.first.value.clone();


                    // reject不需要存通知
                    // let tongzhi = build_hty_tongzhi(
                    //     &notify_type,
                    //     &from_app.app_id,
                    //     None,
                    //     &user_send_to,
                    //     &push_message,
                    //     &first_value,
                    //     &in_role_id.clone(),
                    // )?;
                    //
                    // let _ = HtyTongzhi::create(&tongzhi, conn)?;

                    // todo: 加入通知📢

                    tokio::spawn(async move {
                        let _ = push_wx_message(&to_app, &push_message).await;
                    });
                    // send_tongzhi!(tongzhi, to_app, push_message, db_pool);
                }
                _ => {
                    return Err(anyhow!(HtyErr {
                        code: HtyErrCode::WebErr,
                        reason: Some("invalid notify_type".into())
                    }));
                }
            },
            _ => {
                return Err(anyhow!(HtyErr {
                    code: HtyErrCode::WebErr,
                    reason: Some("notify_type is none".into())
                }));
            }
        }
    }

    Ok(())
}

fn build_hty_tongzhi<T: Serialize>(
    notify_type: &String,
    app_id: &String,
    send_from: Option<HtyUser>,
    send_to: &HtyUser,
    wx_message: &ReqWxPushMessage<T>,
    wx_first_value: &String,
    to_role_id: &Option<String>,
    push_info: &Option<PushInfo>,
) -> anyhow::Result<HtyTongzhi> {
    let meta_content = serde_json::to_string::<ReqWxPushMessage<T>>(&wx_message)?;
    let meta = TongzhiMeta {
        val: Some(meta_content),
    };
    let send_from_real_name = send_from
        .clone()
        .map(|item| item.real_name.unwrap_or_default());
    let content = CommonTongzhiContent {
        to_user: Some(send_to.real_name.clone().unwrap_or_default()),
        from_user: send_from_real_name,
        created_at: Some(current_local_datetime()),
        content: Some(wx_first_value.clone()),
        qumu_sections: None,
        piyue_id: None,
        beizhu: None,
        lianxi_id: None,
        jihua_id: None,
        jihua_start_from: None,
        jihua_end_at: None,
        daka_id: None,
        daka_start_date: None,
        daka_duration_days: None,
    };
    let send_from_hty_id = send_from.clone().map(|item| item.hty_id);
    let hty_tongzhi = HtyTongzhi {
        tongzhi_id: htycommons::uuid(),
        app_id: app_id.clone(),
        tongzhi_type: notify_type.clone(),
        tongzhi_status: String::from(UNREAD),
        send_from: send_from_hty_id,
        send_to: send_to.hty_id.clone(),
        created_at: current_local_datetime(),
        content: Some(content),
        meta: Some(meta),
        role_id: to_role_id.clone(),
        push_info: push_info.clone(),
    };
    Ok(hty_tongzhi)
}
