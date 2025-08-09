-- Your SQL goes here
alter table hty_resources
    alter column res_type type varchar using res_type::varchar;

alter table hty_resources
    alter column res_type drop not null;

