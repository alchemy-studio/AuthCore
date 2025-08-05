alter table hty_action_types rename column hty_action_type_id to hty_label_id;

alter table hty_action_types rename column action_type_name to label_name;

alter table hty_action_types rename column action_type_desc to label_desc;

alter table hty_action_types rename to hty_labels;

create unique index hty_labels_label_name_uindex
    on hty_labels (label_name);

