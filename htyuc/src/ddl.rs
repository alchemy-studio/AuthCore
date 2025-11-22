// use std::ops::DerefMut;
use diesel::PgConnection;
use htycommons::{db, pass_or_panic2};
use htycommons::db::{get_uc_db_url};
use htycommons::common::{APP_STATUS_ACTIVE, current_local_datetime, env_var};
use htycommons::uuid;
use htycommons::cert::{generate_cert_key_pair};
use htyuc_models::models::{HtyApp, HtyLabel, HtyRole, HtyTag, HtyTagRef, AppRole, HtyUser, RoleLabel, UserAppInfo, UserInfoRole};
use tracing::info;
use tracing::debug;

pub fn uc_ddl() {
    let uc_pool = db::pool(&get_uc_db_url());
    debug!("uc_pool -> {:?}", uc_pool.state());

    // let conn = &mut uc_pool.get().unwrap();

    let root_app = insert_root_app(&mut uc_pool.get().expect("Failed to get database connection"));
    let root_tag = insert_root_tag(&mut uc_pool.get().expect("Failed to get database connection"));
    let root_role_id = insert_root_role(&mut uc_pool.get().expect("Failed to get database connection"));

    insert_root_label(&mut uc_pool.get().expect("Failed to get database connection"), &root_role_id.clone());
    insert_root_user(&root_app.app_id, &root_role_id.clone(), &mut uc_pool.get().expect("Failed to get database connection"));

    let ts_app = insert_ts_app(&mut uc_pool.get().expect("Failed to get database connection"), &root_tag.tag_id);
    insert_ts_user(&ts_app.app_id, &mut uc_pool.get().expect("Failed to get database connection"));

    let admin_app = insert_admin_app(&mut uc_pool.get().expect("Failed to get database connection"));
    let admin_role_id = insert_admin_role(&mut uc_pool.get().expect("Failed to get database connection"));
    insert_admin_label(&mut uc_pool.get().expect("Failed to get database connection"), &admin_role_id.clone());
    insert_admin_user(&admin_app.app_id, &admin_role_id.clone(), &mut uc_pool.get().expect("Failed to get database connection"));

}

pub fn insert_root_tag(conn: &mut PgConnection) -> HtyTag {
    let id_tag = uuid();
    let root_tag = HtyTag {
        tag_id: id_tag.clone(),
        tag_name: "SYS_ROOT".to_string(),
        tag_desc: Some("ROOT".to_string()),
        style: None,
    };
    HtyTag::create(&root_tag, conn).ok();
    root_tag
}

pub fn insert_root_role(conn: &mut PgConnection) -> String {
    let role_id = uuid();

    let role_root = HtyRole {
        hty_role_id: role_id.clone(),
        role_key: "ROOT".to_string(),
        role_desc: Some("ROOT".to_string()),
        role_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
        role_name: None,
    };

    let _root_role = HtyRole::create(&role_root, conn)
        .expect("Failed to create root role");

    _root_role.hty_role_id
}

pub fn insert_admin_role(conn: &mut PgConnection) -> String {
    let id_role_admin = uuid();

    let role_admin = HtyRole {
        hty_role_id: id_role_admin.clone(),
        role_key: "ADMIN".to_string(),
        role_desc: Some("管理员".to_string()),
        role_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
        role_name: None,
    };

    let _role_admin_c = HtyRole::create(&role_admin, conn)
        .expect("Failed to create admin role");

    _role_admin_c.hty_role_id
}


pub fn insert_tester_role(app_id: &String, conn: &mut PgConnection) -> String {
    let id_role_tester = uuid();

    let role_tester = HtyRole {
        hty_role_id: id_role_tester.clone(),
        role_key: "TESTER".to_string(),
        role_desc: Some("测试员".to_string()),
        role_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
        role_name: None,
    };

    let created_role_tester = HtyRole::create(&role_tester, conn)
        .expect("Failed to create tester role");


    let tester_app_role = AppRole {
        the_id: uuid(),
        app_id: app_id.clone(),
        role_id: created_role_tester.hty_role_id.clone(),
    };

    AppRole::create(&tester_app_role, conn).ok();

    created_role_tester.hty_role_id
}

pub fn insert_root_label(conn: &mut PgConnection, role_id: &String) {
    let id_root_label = uuid().to_owned();

    let root_label = HtyLabel {
        hty_label_id: id_root_label.clone(),
        label_name: "SYS_ROOT".to_string(),
        label_desc: Some("SYS_ROOT".to_string()),
        label_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
    };

    let _root_label_c = HtyLabel::create(&root_label, conn);

    let role_label = RoleLabel {
        the_id: uuid(),
        role_id: role_id.clone(),
        label_id: id_root_label.clone(),
    };

    let _role_label_c = RoleLabel::create(&role_label, conn);
}

pub fn insert_admin_label(conn: &mut PgConnection, role_id: &String) {
    let id_admin_label = uuid().to_owned();

    let admin_label = HtyLabel {
        hty_label_id: id_admin_label.clone(),
        label_name: "SYS_ADMIN".to_string(),
        label_desc: Some("SYS_ADMIN".to_string()),
        label_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
    };

    let _admin_label_c = HtyLabel::create(&admin_label, conn);

    let role_label = RoleLabel {
        the_id: uuid(),
        role_id: role_id.clone(),
        label_id: id_admin_label.clone(),
    };

    let _role_label_c = RoleLabel::create(&role_label, conn);
}

pub fn insert_tester_label(conn: &mut PgConnection, role_id: &String) {
    let id_tester_label = uuid().to_owned();

    let tester_label = HtyLabel {
        hty_label_id: id_tester_label.clone(),
        label_name: "SYS_TESTER".to_string(),
        label_desc: Some("SYS_TESTER".to_string()),
        label_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
    };

    let _tester_label_c = HtyLabel::create(&tester_label, conn);

    let role_label = RoleLabel {
        the_id: uuid(),
        role_id: role_id.clone(),
        label_id: id_tester_label.clone(),
    };

    let _role_label_c = RoleLabel::create(&role_label, conn);
}


pub fn insert_ts_app(conn: &mut PgConnection, tag_id: &String) -> HtyApp {
    info!("insert task server app...");
    let app_key_pair = generate_cert_key_pair()
        .expect("Failed to generate cert key pair");

    let app_id = uuid();
    let pubkey = app_key_pair.pubkey
        .ok_or_else(|| anyhow::anyhow!("pubkey is missing"))
        .expect("pubkey is required");
    let privkey = app_key_pair.privkey
        .ok_or_else(|| anyhow::anyhow!("privkey is missing"))
        .expect("privkey is required");
    let ts_app = HtyApp {
        app_id: app_id.clone(),
        wx_secret: Some("".to_string()),
        domain: env_var("TS_DOMAIN"),
        app_desc: Some("task server app".to_string()),
        app_status: APP_STATUS_ACTIVE.to_string(),
        pubkey: Some(pubkey),
        privkey: Some(privkey),
        wx_id: None,
        is_wx_app: Some(false),
    };

    HtyApp::create(&ts_app, conn).ok();

    // todo replace with enum type for `ref_type`
    let ts_app_tag_ref = HtyTagRef {
        the_id: uuid(),
        hty_tag_id: tag_id.clone(),
        ref_id: app_id.clone(),
        ref_type: "APP".to_string(),
        meta: None,
    };

    HtyTagRef::create(&ts_app_tag_ref, conn).ok();

    ts_app
}

pub fn insert_ts_user(app_id: &String, conn: &mut PgConnection) -> HtyUser {
    let id_music_room_user_dev = uuid();
    let ts_user = HtyUser {
        hty_id: id_music_room_user_dev.clone(),
        union_id: Some("MOCKED_TS_USER_UNION_ID".to_string()),
        enabled: false,
        created_at: Some(current_local_datetime()),
        real_name: Some("MOCKED_TS_USER".to_string()),
        sex: Some(1),
        mobile: None,
        settings: None,
    };

    let ts_user_info = UserAppInfo {
        hty_id: id_music_room_user_dev.clone(),
        app_id: Some(app_id.clone()),
        openid: Some("weli_open_id".to_string()),
        is_registered: false,
        id: uuid(),
        username: Some("weli".to_string()),
        password: None,
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    info!("creating task server user weli...");
    pass_or_panic2(HtyUser::create_with_info(&ts_user, &Some(ts_user_info), conn));

    ts_user
}

pub fn insert_wx_gongzhonghao(conn: &mut PgConnection) -> HtyApp {
    info!("insert wx mp app...");


    let app_key_pair = generate_cert_key_pair()
        .expect("Failed to generate cert key pair for wx mp app");

    let wx_mp_app = HtyApp {
        app_id: uuid(),
        domain: env_var("WX_MP_DOMAIN"),
        app_desc: Some("微信公众号APP，用于记录公众号信息".to_string()),
        app_status: APP_STATUS_ACTIVE.to_string().clone(),
        pubkey: app_key_pair.pubkey.clone(),
        privkey: app_key_pair.privkey.clone(),
        wx_id: env_var("WX_MP_ID"),
        wx_secret: env_var("WX_MP_SECRET"),
        is_wx_app: Some(true),
    };

    HtyApp::create(&wx_mp_app, conn).ok();
    info!("inserted wx app... {:?}", &wx_mp_app);
    wx_mp_app
}

pub fn insert_root_app(conn: &mut PgConnection) -> HtyApp {
    info!("insert root app...");
    let app_key_pair = generate_cert_key_pair()
        .expect("Failed to generate cert key pair for root app");

    let root_app = HtyApp {
        app_id: uuid(),
        domain: Some("root".to_string()),
        app_desc: Some("virtual root app".to_string()),
        app_status: APP_STATUS_ACTIVE.to_string().clone(),
        pubkey: app_key_pair.pubkey.clone(),
        privkey: app_key_pair.privkey.clone(),
        wx_id: None,
        wx_secret: Some("default secret".to_string()),
        is_wx_app: Some(false),
    };

    HtyApp::create(&root_app, conn).ok();

    root_app
}

pub fn insert_root_user(app_id: &String, role_id: &String, conn: &mut PgConnection) -> HtyUser {
    let root_id = uuid();

    let root_user = HtyUser {
        hty_id: root_id.clone(),
        union_id: None,
        enabled: true,
        created_at: Some(current_local_datetime()),
        real_name: Some("ROOT".to_string()),
        sex: None,
        mobile: None,
        settings: None,
    };

    let root_user_info = UserAppInfo {
        hty_id: root_id.clone(),
        app_id: Some(app_id.clone()),
        openid: None,
        is_registered: true,
        id: uuid(),
        username: Some("root".to_string()),
        password: None,
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    debug!("insert root user app_id -> {}", app_id.clone());
    let app_id_str = root_user_info.app_id.as_ref()
        .map(|id| id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    debug!("insert root user info -> app_id: {}, hty_id: {}", app_id_str, root_id.clone());

    pass_or_panic2(HtyUser::create_with_info(&root_user, &Some(root_user_info), conn));
    let user_info = UserAppInfo::find_by_hty_id_and_app_id(&root_id, &app_id, conn)
        .expect("Failed to find user info");
    let user_info_role = UserInfoRole {
        the_id: uuid(),
        user_info_id: user_info.id.clone(),
        role_id: role_id.clone(),
    };

    debug!("insert user info role -> id={},role_id={},u_id={}", user_info_role.the_id.clone(), user_info_role.role_id.clone(), user_info_role.user_info_id.clone());
    let _user_info_role_c = UserInfoRole::create(&user_info_role, conn);

    root_user
}

pub fn insert_admin_app(conn: &mut PgConnection) -> HtyApp {
    info!("insert admin app...");
    let app_key_pair = generate_cert_key_pair()
        .expect("Failed to generate cert key pair for admin app");

    let admin_app = HtyApp {
        app_id: uuid(),
        domain: env_var("ADMIN_DOMAIN"),
        app_desc: Some("admin system".to_string()),
        app_status: APP_STATUS_ACTIVE.to_string().clone(),
        pubkey: app_key_pair.pubkey.clone(),
        privkey: app_key_pair.privkey.clone(),
        wx_id: None,
        wx_secret: Some("default secret".to_string()),
        is_wx_app: Some(false),
    };

    HtyApp::create(&admin_app, conn).ok();

    admin_app
}

pub fn insert_admin_user(app_id: &String, role_id: &String, conn: &mut PgConnection) -> HtyUser {
    let id_admin = uuid();

    let admin_user = HtyUser {
        hty_id: id_admin.clone(),
        union_id: None,
        enabled: true,
        created_at: Some(current_local_datetime()),
        real_name: Some("ADMIN".to_string()),
        sex: None,
        mobile: None,
        settings: None,
    };

    let admin_user_info = UserAppInfo {
        hty_id: id_admin.clone(),
        app_id: Some(app_id.clone()),
        openid: None,
        is_registered: true,
        id: uuid(),
        username: Some("admin".to_string()),
        password: Some("123".to_string()),
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    debug!("insert admin user app_id -> {}", app_id.clone());
    if let Some(app_id_ref) = admin_user_info.app_id.as_ref() {
        debug!("insert admin user app_id user info ->  {}", app_id_ref.clone());
    }

    info!("creating admin user...");

    pass_or_panic2(HtyUser::create_with_info(
        &admin_user,
        &Some(admin_user_info),
        conn,
    ));
    let user_info = UserAppInfo::find_by_hty_id_and_app_id(&id_admin, &app_id, conn)
        .expect("Failed to find admin user info after creation");
    let user_info_role = UserInfoRole {
        the_id: uuid(),
        user_info_id: user_info.id.clone(),
        role_id: role_id.clone(),
    };

    debug!("insert user info role -> id={},role_id={},u_id={}", user_info_role.the_id.clone(), user_info_role.role_id.clone(), user_info_role.user_info_id.clone());
    let _user_info_role_c = UserInfoRole::create(&user_info_role, conn);

    admin_user
}

pub fn insert_teacher_role(app_id: &String, conn: &mut PgConnection) -> HtyRole {
    let teacher_role_id = uuid().to_owned();
    let teacher_role = HtyRole {
        hty_role_id: teacher_role_id.clone(),
        role_key: "TEACHER".to_string(),
        role_desc: Some("教师".to_string()),
        role_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
        role_name: None,
    };

    HtyRole::create(&teacher_role, conn).ok();

    let teacher_app_role = AppRole {
        the_id: uuid(),
        app_id: app_id.clone(),
        role_id: teacher_role_id.clone(),
    };
    AppRole::create(&teacher_app_role, conn).ok();

    teacher_role
}

pub fn insert_student_role(app_id: &String, conn: &mut PgConnection) -> HtyRole {
    let student_role_id = uuid().to_owned();
    let student_role = HtyRole {
        hty_role_id: student_role_id.clone(),
        role_key: "STUDENT".to_string(),
        role_desc: Some("学生".to_string()),
        role_status: APP_STATUS_ACTIVE.to_string(),
        style: None,
        role_name: None,
    };

    HtyRole::create(&student_role, conn).ok();

    let student_app_role = AppRole {
        the_id: uuid(),
        app_id: app_id.clone(),
        role_id: student_role_id.clone(),
    };

    AppRole::create(&student_app_role, conn).ok();

    student_role
}

pub fn insert_music_room_teacher(app_id: &String, role_id: Option<&String>, conn: &mut PgConnection) -> HtyUser {
    let teacher = HtyUser {
        hty_id: uuid(),
        union_id: None,
        enabled: true,
        created_at: Some(current_local_datetime()),
        real_name: Some("MOCKED_TEACHER".to_string()),
        sex: Some(1),
        mobile: None,
        settings: None,
    };

    let teacher_app_info = UserAppInfo {
        hty_id: teacher.hty_id.clone(),
        app_id: Some(app_id.clone()),
        openid: None,
        is_registered: true,
        id: uuid(),
        username: Some("moicen".to_string()),
        password: None,
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    info!("creating teacher moicen for music room...");
    pass_or_panic2(HtyUser::create_with_info(&teacher, &Some(teacher_app_info), conn));

    if let Some(role_id_val) = role_id {
        let user_info = UserAppInfo::find_by_hty_id_and_app_id(&teacher.hty_id.clone(), &app_id, conn)
            .expect("Failed to find user info");
        let user_info_role = UserInfoRole {
            the_id: uuid(),
            user_info_id: user_info.id.clone(),
            role_id: role_id_val.clone(),
        };
        UserInfoRole::create(&user_info_role, conn).ok();
    }

    teacher
}

pub fn insert_music_room_student(app_id: &String, role_id: &String, real_name: String, username: String, conn: &mut PgConnection) -> HtyUser {
    let student_user = HtyUser {
        hty_id: uuid(),
        union_id: None,
        enabled: true,
        created_at: Some(current_local_datetime()),
        real_name: Some(real_name.to_string()),
        sex: Some(1),
        mobile: None,
        settings: None,
    };

    let student_user_info = UserAppInfo {
        hty_id: student_user.hty_id.clone(),
        app_id: Some(app_id.clone()),
        openid: None,
        is_registered: true,
        id: uuid(),
        username: Some(username.to_string()),
        password: None,
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    info!("creating student {}...", real_name);
    pass_or_panic2(HtyUser::create_with_info(&student_user, &Some(student_user_info), conn));
    let user_info = UserAppInfo::find_by_hty_id_and_app_id(&student_user.hty_id.clone(), &app_id.clone(), conn)
        .expect("Failed to find student user info after creation");
    let user_info_role = UserInfoRole {
        the_id: uuid(),
        user_info_id: user_info.id.clone(),
        role_id: role_id.clone(),
    };

    UserInfoRole::create(&user_info_role, conn).ok();
    student_user
}


pub fn insert_role_tester_user(app_id: &String, role_id: &String, conn: &mut PgConnection) -> HtyUser {
    let id_tester = uuid();

    let tester_user = HtyUser {
        hty_id: id_tester.clone(),
        union_id: None,
        enabled: true,
        created_at: Some(current_local_datetime()),
        real_name: Some("TESTER".to_string()),
        sex: None,
        mobile: None,
        settings: None,
    };

    let tester_user_info = UserAppInfo {
        hty_id: id_tester.clone(),
        app_id: Some(app_id.clone()),
        openid: None,
        is_registered: true,
        id: uuid(),
        username: Some("tester".to_string()),
        password: Some("tester".to_string()),
        meta: None,
        created_at: Some(current_local_datetime()),
        teacher_info: None,
        student_info: None,
        reject_reason: None,
        needs_refresh: Some(false),
        avatar_url: None,
    };

    debug!("insert tester user app_id -> {}", app_id.clone());
    if let Some(app_id_ref) = tester_user_info.app_id.as_ref() {
        debug!("insert tester user app_id user info ->  {}", app_id_ref.clone());
    }

    info!("creating tester user...");

    pass_or_panic2(HtyUser::create_with_info(
        &tester_user,
        &Some(tester_user_info),
        conn,
    ));
    let user_info = UserAppInfo::find_by_hty_id_and_app_id(&id_tester, &app_id, conn)
        .expect("Failed to find tester user info after creation");
    let user_info_role = UserInfoRole {
        the_id: uuid(),
        user_info_id: user_info.id.clone(),
        role_id: role_id.clone(),
    };

    debug!("insert user info role -> id={},role_id={},u_id={}", user_info_role.the_id.clone(), user_info_role.role_id.clone(), user_info_role.user_info_id.clone());
    let _user_info_role_c = UserInfoRole::create(&user_info_role, conn);

    tester_user
}