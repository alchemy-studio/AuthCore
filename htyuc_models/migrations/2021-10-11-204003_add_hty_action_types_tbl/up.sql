create table hty_action_types
(
    hty_action_type_id varchar not null,
    action_type_name varchar not null,
    action_type_desc varchar
);

create unique index hty_action_types_action_type_name_uindex
    on hty_action_types (action_type_name);

create unique index hty_action_types_hty_action_type_id_uindex
    on hty_action_types (hty_action_type_id);

alter table hty_action_types
    add constraint hty_action_types_pk
        primary key (hty_action_type_id);

