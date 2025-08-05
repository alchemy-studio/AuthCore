-- Your SQL goes here

create table hty_user_group
(
    id         varchar not null,
    user_ids   jsonb,
    group_type varchar not null,
    created_at varchar,
    created_by varchar
);

create unique index hty_user_group_id_uindex
    on hty_user_group (id);

alter table hty_user_group
    add constraint hty_user_group_pk
        primary key (id);

