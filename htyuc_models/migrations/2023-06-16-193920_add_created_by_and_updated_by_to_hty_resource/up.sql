-- Your SQL goes here
alter table hty_resources
    add updated_at timestamp;

alter table hty_resources
    add updated_by varchar;

