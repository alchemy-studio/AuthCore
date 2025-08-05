-- Your SQL goes here
create table hty_tag_refs
(
    the_id varchar not null,
    hty_tag_id varchar not null,
    ref_id varchar not null,
    ref_type varchar not null
);

create unique index hty_tag_refs_the_id_uindex
    on hty_tag_refs (the_id);

alter table hty_tag_refs
    add constraint hty_tag_refs_pk
        primary key (the_id);

