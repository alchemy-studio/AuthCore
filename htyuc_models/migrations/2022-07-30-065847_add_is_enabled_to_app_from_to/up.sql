-- Your SQL goes here

alter table app_from_to
    add is_enabled boolean;

update app_from_to set is_enabled=true;