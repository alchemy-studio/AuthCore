-- Your SQL goes here
alter table user_app_info alter column user_role type varchar using user_role::varchar;

