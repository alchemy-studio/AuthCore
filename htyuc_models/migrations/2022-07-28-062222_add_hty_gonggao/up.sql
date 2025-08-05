-- Your SQL goes here

create table hty_gonggao
(
    id             varchar   not null,
    app_id         varchar,
    created_at     timestamp not null,
    gonggao_status varchar,
    content        varchar
);

create unique index hty_gonggao_id_uindex
    on hty_gonggao (id);

alter table hty_gonggao
    add constraint hty_gonggao_pk
        primary key (id);

