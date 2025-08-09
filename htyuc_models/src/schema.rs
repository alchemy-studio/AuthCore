// @generated automatically by Diesel CLI.

diesel::table! {
    actions_labels (the_id) {
        the_id -> Varchar,
        action_id -> Varchar,
        label_id -> Varchar,
    }
}

diesel::table! {
    app_from_to (id) {
        id -> Varchar,
        from_app_id -> Varchar,
        to_app_id -> Varchar,
        is_enabled -> Bool,
    }
}

diesel::table! {
    apps_roles (the_id) {
        the_id -> Varchar,
        app_id -> Varchar,
        role_id -> Varchar,
    }
}

diesel::table! {
    hty_actions (hty_action_id) {
        hty_action_id -> Varchar,
        action_name -> Varchar,
        action_desc -> Nullable<Varchar>,
        action_status -> Varchar,
    }
}

diesel::table! {
    hty_apps (app_id) {
        app_id -> Varchar,
        wx_secret -> Nullable<Varchar>,
        domain -> Nullable<Varchar>,
        app_desc -> Nullable<Varchar>,
        app_status -> Varchar,
        pubkey -> Nullable<Varchar>,
        privkey -> Nullable<Varchar>,
        wx_id -> Nullable<Varchar>,
        is_wx_app -> Nullable<Bool>,
    }
}

diesel::table! {
    hty_gonggao (id) {
        id -> Varchar,
        app_id -> Nullable<Varchar>,
        created_at -> Timestamp,
        gonggao_status -> Nullable<Varchar>,
        content -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_labels (hty_label_id) {
        hty_label_id -> Varchar,
        label_name -> Varchar,
        label_desc -> Nullable<Varchar>,
        label_status -> Varchar,
        style -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_resources (hty_resource_id) {
        filename -> Nullable<Varchar>,
        app_id -> Varchar,
        hty_resource_id -> Varchar,
        created_at -> Nullable<Timestamp>,
        url -> Varchar,
        res_type -> Nullable<Varchar>,
        created_by -> Nullable<Varchar>,
        tasks -> Nullable<Jsonb>,
        compress_processed -> Nullable<Bool>,
        updated_at -> Nullable<Timestamp>,
        updated_by -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_roles (hty_role_id) {
        hty_role_id -> Varchar,
        role_key -> Varchar,
        role_desc -> Nullable<Varchar>,
        role_status -> Varchar,
        style -> Nullable<Varchar>,
        role_name -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_tag_refs (the_id) {
        the_id -> Varchar,
        hty_tag_id -> Varchar,
        ref_id -> Varchar,
        ref_type -> Varchar,
        meta -> Nullable<Jsonb>,
    }
}

diesel::table! {
    hty_tags (tag_id) {
        tag_id -> Varchar,
        tag_name -> Varchar,
        tag_desc -> Nullable<Varchar>,
        style -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_template (id) {
        id -> Varchar,
        template_key -> Varchar,
        created_at -> Timestamp,
        created_by -> Varchar,
        template_desc -> Nullable<Varchar>,
    }
}

diesel::table! {
    hty_template_data (id) {
        id -> Varchar,
        app_id -> Varchar,
        template_id -> Varchar,
        template_val -> Nullable<Varchar>,
        template_text -> Nullable<Jsonb>,
        created_at -> Timestamp,
        created_by -> Varchar,
    }
}

diesel::table! {
    hty_tongzhi (tongzhi_id) {
        tongzhi_id -> Varchar,
        app_id -> Varchar,
        tongzhi_type -> Varchar,
        tongzhi_status -> Varchar,
        send_from -> Nullable<Varchar>,
        send_to -> Varchar,
        created_at -> Timestamp,
        content -> Nullable<Jsonb>,
        meta -> Nullable<Jsonb>,
        role_id -> Nullable<Varchar>,
        push_info -> Nullable<Jsonb>,
    }
}

diesel::table! {
    hty_user_group (id) {
        id -> Varchar,
        users -> Nullable<Jsonb>,
        group_type -> Varchar,
        created_at -> Nullable<Timestamp>,
        created_by -> Nullable<Varchar>,
        app_id -> Varchar,
        group_name -> Varchar,
        is_delete -> Bool,
        group_desc -> Nullable<Varchar>,
        parent_id -> Nullable<Varchar>,
        owners -> Nullable<Jsonb>,
    }
}

diesel::table! {
    hty_user_rels (id) {
        id -> Varchar,
        from_user_id -> Varchar,
        to_user_id -> Varchar,
        rel_type -> Varchar,
    }
}

diesel::table! {
    hty_users (hty_id) {
        hty_id -> Varchar,
        union_id -> Nullable<Varchar>,
        enabled -> Bool,
        created_at -> Nullable<Timestamp>,
        real_name -> Nullable<Varchar>,
        sex -> Nullable<Int4>,
        mobile -> Nullable<Varchar>,
        settings -> Nullable<Jsonb>,
    }
}

diesel::table! {
    hty_visitors (id) {
        hty_id -> Nullable<Int4>,
        id -> Varchar,
        meta -> Jsonb,
        last_logged_at -> Timestamp,
    }
}

diesel::table! {
    roles_actions (the_id) {
        the_id -> Varchar,
        role_id -> Varchar,
        action_id -> Varchar,
    }
}

diesel::table! {
    roles_labels (the_id) {
        the_id -> Varchar,
        role_id -> Varchar,
        label_id -> Varchar,
    }
}

diesel::table! {
    user_app_info (id) {
        hty_id -> Varchar,
        app_id -> Nullable<Varchar>,
        openid -> Nullable<Varchar>,
        is_registered -> Bool,
        id -> Varchar,
        username -> Nullable<Varchar>,
        password -> Nullable<Varchar>,
        meta -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamp>,
        teacher_info -> Nullable<Jsonb>,
        student_info -> Nullable<Jsonb>,
        reject_reason -> Nullable<Varchar>,
        needs_refresh -> Nullable<Bool>,
        avatar_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_info_roles (the_id) {
        the_id -> Varchar,
        user_info_id -> Varchar,
        role_id -> Varchar,
    }
}

diesel::joinable!(actions_labels -> hty_actions (action_id));
diesel::joinable!(actions_labels -> hty_labels (label_id));
diesel::joinable!(apps_roles -> hty_apps (app_id));
diesel::joinable!(apps_roles -> hty_roles (role_id));
diesel::joinable!(hty_gonggao -> hty_apps (app_id));
diesel::joinable!(hty_resources -> hty_apps (app_id));
diesel::joinable!(hty_resources -> hty_users (created_by));
diesel::joinable!(hty_tag_refs -> hty_tags (hty_tag_id));
diesel::joinable!(hty_template_data -> hty_apps (app_id));
diesel::joinable!(hty_template_data -> hty_template (template_id));
diesel::joinable!(hty_tongzhi -> hty_apps (app_id));
diesel::joinable!(hty_tongzhi -> hty_roles (role_id));
diesel::joinable!(hty_user_group -> hty_apps (app_id));
diesel::joinable!(roles_actions -> hty_actions (action_id));
diesel::joinable!(roles_actions -> hty_roles (role_id));
diesel::joinable!(roles_labels -> hty_labels (label_id));
diesel::joinable!(roles_labels -> hty_roles (role_id));
diesel::joinable!(user_app_info -> hty_apps (app_id));
diesel::joinable!(user_app_info -> hty_users (hty_id));
diesel::joinable!(user_info_roles -> hty_roles (role_id));
diesel::joinable!(user_info_roles -> user_app_info (user_info_id));

diesel::allow_tables_to_appear_in_same_query!(
    actions_labels,
    app_from_to,
    apps_roles,
    hty_actions,
    hty_apps,
    hty_gonggao,
    hty_labels,
    hty_resources,
    hty_roles,
    hty_tag_refs,
    hty_tags,
    hty_template,
    hty_template_data,
    hty_tongzhi,
    hty_user_group,
    hty_user_rels,
    hty_users,
    hty_visitors,
    roles_actions,
    roles_labels,
    user_app_info,
    user_info_roles,
);
