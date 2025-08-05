-- Your SQL goes here

alter table hty_user_group
    rename column user_ids to users;

alter table hty_user_group
    add group_name varchar not null;

alter table hty_user_group
    add is_delete boolean not null;

alter table hty_user_group
    add group_desc varchar;

alter table hty_user_group
    add parent_id varchar;

alter table hty_user_group
    add constraint hty_user_group_hty_user_group_id_fk
        foreign key (parent_id) references hty_user_group;

