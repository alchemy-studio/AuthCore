-- Your SQL goes here

create table hty_tags
(
    tag_id varchar not null,
    tag_name varchar not null,
    tag_type varchar not null,
    tag_desc varchar
);

create unique index hty_tags_tag_id_uindex
    on hty_tags (tag_id);

create unique index hty_tags_tag_name_uindex
    on hty_tags (tag_name);

create unique index hty_tags_tag_type_uindex
    on hty_tags (tag_type);

alter table hty_tags
    add constraint hty_tags_pk
        primary key (tag_id);

