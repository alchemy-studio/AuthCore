alter table user_info_roles
    add app_id varchar not null;

alter table user_info_roles
    add constraint user_info_roles_hty_apps_app_id_fk
        foreign key (app_id) references hty_apps;

