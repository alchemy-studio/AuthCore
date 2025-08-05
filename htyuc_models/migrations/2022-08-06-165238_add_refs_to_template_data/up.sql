-- Your SQL goes here

alter table hty_template_data
    add constraint hty_template_data_hty_apps_app_id_fk
        foreign key (app_id) references hty_apps;

alter table hty_template_data
    add constraint hty_template_data_hty_template_id_fk
        foreign key (template_id) references hty_template;

