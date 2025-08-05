-- Your SQL goes here

alter table hty_tag_refs
    add constraint hty_tag_refs_hty_tags_tag_id_fk
        foreign key (hty_tag_id) references hty_tags;

