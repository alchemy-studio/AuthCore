-- Your SQL goes here
alter table hty_user_group
    alter column created_at type timestamp using created_at::timestamp;

