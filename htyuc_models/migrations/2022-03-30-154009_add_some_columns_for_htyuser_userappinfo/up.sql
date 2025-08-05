-- Your SQL goes here
alter table hty_users
    add sex int;

alter table hty_users
    add mobile varchar;

alter table user_app_info
    add teacher_info jsonb;

alter table user_app_info
    add student_info jsonb;