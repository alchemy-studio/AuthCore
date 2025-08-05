-- Your SQL goes here
alter table hty_roles
    add role_status varchar not null;
alter table hty_actions
    add action_status varchar not null;
alter table hty_labels
    add label_status varchar not null;