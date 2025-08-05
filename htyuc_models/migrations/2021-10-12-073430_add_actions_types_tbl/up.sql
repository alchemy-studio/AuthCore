create table actions_types
(
    the_id varchar not null,
    action_id varchar not null
        constraint actions_types_hty_actions_hty_action_id_fk
            references hty_actions,
    type_id varchar not null
        constraint actions_types_hty_action_types_hty_action_type_id_fk
            references hty_action_types
);

create unique index actions_types_the_id_uindex
    on actions_types (the_id);

alter table actions_types
    add constraint actions_types_pk
        primary key (the_id);

