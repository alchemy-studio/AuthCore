-- Your SQL goes here

create table hty_tongzhi
(
    tongzhi_id     varchar   not null
        constraint hty_tongzhi_pk
            primary key,
    app_id         varchar   not null,
    tongzhi_type   varchar   not null,
    tongzhi_status varchar   not null,
    send_from      varchar   not null,
    send_to        varchar   not null,
    created_at     timestamp not null,
    content        jsonb,
    meta           jsonb
);

create unique index hty_tongzhi_tongzhi_id_uindex
    on hty_tongzhi (tongzhi_id);

