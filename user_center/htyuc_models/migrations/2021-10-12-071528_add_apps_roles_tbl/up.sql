create table apps_roles
(
    the_id varchar not null,
    app_id varchar not null
        constraint apps_roles_hty_apps_app_id_fk
            references hty_apps,
    role_id varchar not null
        constraint apps_roles_hty_roles_hty_role_id_fk
            references hty_roles
);

create unique index apps_roles_id_uindex
    on apps_roles (the_id);

alter table apps_roles
    add constraint apps_roles_pk
        primary key (the_id);
