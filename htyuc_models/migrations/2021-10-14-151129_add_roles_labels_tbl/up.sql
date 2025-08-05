create table roles_labels
(
    the_id varchar not null,
    role_id varchar not null
        constraint roles_labels_hty_roles_hty_role_id_fk
            references hty_roles,
    label_id varchar not null
        constraint roles_labels_hty_labels_hty_label_id_fk
            references hty_labels
);

create unique index roles_labels_the_id_uindex
    on roles_labels (the_id);

alter table roles_labels
    add constraint roles_labels_pk
        primary key (the_id);

