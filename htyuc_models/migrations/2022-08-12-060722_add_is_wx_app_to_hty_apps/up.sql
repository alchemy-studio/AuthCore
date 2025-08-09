-- Your SQL goes here

alter table hty_apps
    add is_wx_app boolean default false;

-- update hty_apps set is_wx_app=false;
