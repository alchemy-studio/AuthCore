-- Your SQL goes here

alter table hty_roles
    rename column role_name to role_key;

create unique index hty_roles_role_key_uindex
    on hty_roles (role_key);

