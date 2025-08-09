-- Your SQL goes here

alter table hty_user_group
    add app_id varchar not null;

alter table hty_user_group
    add constraint hty_user_group_hty_apps_app_id_fk
        foreign key (app_id) references hty_apps;

