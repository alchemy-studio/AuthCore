-- Your SQL goes here


alter table hty_gonggao
    add constraint hty_gonggao_hty_apps_app_id_fk
        foreign key (app_id) references hty_apps;

