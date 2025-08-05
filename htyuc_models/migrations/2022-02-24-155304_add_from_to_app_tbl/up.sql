-- Your SQL goes here
create table app_from_to
(
    id          varchar not null
        constraint app_from_to_pk
            primary key,
    from_app_id varchar not null
        constraint app_from_to_hty_apps_app_id_fk
            references hty_apps (app_id),
    to_app_id   varchar not null
        constraint app_from_to_hty_apps_app_id_fk_2
            references hty_apps (app_id)
);

create unique index app_from_to_id_uindex
    on app_from_to (id);

