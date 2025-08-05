-- Your SQL goes here

alter table hty_tongzhi
    add constraint hty_tongzhi_hty_users_hty_id_fk
        foreign key (send_from) references hty_users;

alter table hty_tongzhi
    add constraint hty_tongzhi_hty_users_hty_id_fk_2
        foreign key (send_to) references hty_users;

