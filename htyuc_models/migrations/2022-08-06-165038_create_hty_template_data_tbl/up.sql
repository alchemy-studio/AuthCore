-- Your SQL goes here

create table hty_template_data
(
    id            varchar   not null,
    app_id        varchar   not null,
    template_id   varchar   not null,
    template_val  varchar,
    template_text jsonb,
    created_at    timestamp not null,
    created_by    varchar   not null
);

create unique index hty_template_data_id_uindex
    on hty_template_data (id);

alter table hty_template_data
    add constraint hty_template_data_pk
        primary key (id);

