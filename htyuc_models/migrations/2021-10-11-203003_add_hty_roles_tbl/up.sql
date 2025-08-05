create table hty_roles
(
    hty_role_id varchar not null,
    role_name varchar not null,
    role_desc varchar
);

create unique index hty_roles_hty_role_id_uindex
    on hty_roles (hty_role_id);

create unique index hty_roles_role_name_uindex
    on hty_roles (role_name);

alter table hty_roles
    add constraint hty_roles_pk
        primary key (hty_role_id);

