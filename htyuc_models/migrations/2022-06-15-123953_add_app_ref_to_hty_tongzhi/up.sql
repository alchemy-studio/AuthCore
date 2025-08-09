-- Your SQL goes here

alter table hty_tongzhi
    add constraint hty_tongzhi_hty_apps_app_id_fk
        foreign key (app_id) references hty_apps;

