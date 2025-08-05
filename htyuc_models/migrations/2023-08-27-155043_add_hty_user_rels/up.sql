-- Your SQL goes here

create table hty_user_rels
(
    id           varchar not null
        constraint hty_user_rels_pk
            primary key,
    from_user_id varchar not null,
    to_user_id   varchar not null,
    rel_type     varchar not null
);

comment on column hty_user_rels.rel_type is 'user relationship type, for example: STUDENT / TEACHER, etc ';

