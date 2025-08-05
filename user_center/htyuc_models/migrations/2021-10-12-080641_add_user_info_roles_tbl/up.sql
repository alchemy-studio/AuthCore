create table user_info_roles
(
    the_id varchar not null,
    user_info_id varchar not null
        constraint user_info_roles_user_app_info_id_fk
            references user_app_info,
    role_id varchar not null
        constraint user_info_roles_hty_roles_hty_role_id_fk
            references hty_roles
);

create unique index user_info_roles_the_id_uindex
    on user_info_roles (the_id);

alter table user_info_roles
    add constraint user_info_roles_pk
        primary key (the_id);

