-- Your SQL goes here

alter table hty_tongzhi
    add role_id varchar;

alter table hty_tongzhi
    add constraint hty_tongzhi_hty_roles_hty_role_id_fk
        foreign key (role_id) references hty_roles;

