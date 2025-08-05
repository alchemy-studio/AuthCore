-- Your SQL goes here

create table hty_template
(
    id           varchar   not null,
    template_key varchar   not null,
    created_at   timestamp not null,
    created_by   varchar   not null,
    "desc"       varchar
);

create unique index hty_template_id_uindex
    on hty_template (id);

create unique index hty_template_template_key_uindex
    on hty_template (template_key);

alter table hty_template
    add constraint hty_template_pk
        primary key (id);

