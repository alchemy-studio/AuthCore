create table hty_actions
(
    hty_action_id varchar not null,
    action_name varchar not null,
    action_desc varchar
);

create unique index hty_actions_action_name_uindex
    on hty_actions (action_name);

create unique index hty_actions_hty_action_id_uindex
    on hty_actions (hty_action_id);

alter table hty_actions
    add constraint hty_actions_pk
        primary key (hty_action_id);

