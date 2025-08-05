-- Your SQL goes here

drop index hty_tags_tag_type_uindex;

alter table hty_tags drop column tag_type;
