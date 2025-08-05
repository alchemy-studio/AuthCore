create table roles_actions
(
    the_id varchar not null,
    role_id varchar not null
        constraint roles_actions_hty_roles_hty_role_id_fk
            references hty_roles,
    action_id varchar not null
        constraint roles_actions_hty_actions_hty_action_id_fk
            references hty_actions
);

create unique index roles_actions_the_id_uindex
    on roles_actions (the_id);

alter table roles_actions
    add constraint roles_actions_pk
        primary key (the_id);

