create table organizations
(
    id varchar not null,
    app_id varchar not null
        constraint organizations_hty_apps_app_id_fk
            references hty_apps (app_id),
    org_name varchar not null,
    org_desc varchar,
    homepage_md text,
    org_status varchar not null default 'ACTIVE',
    created_at timestamp not null default now(),
    created_by varchar
        constraint organizations_hty_users_created_by_fk
            references hty_users (hty_id),
    updated_at timestamp,
    updated_by varchar
        constraint organizations_hty_users_updated_by_fk
            references hty_users (hty_id),
    is_delete boolean not null default false
);

create unique index organizations_id_uindex
    on organizations (id);

alter table organizations
    add constraint organizations_pk
        primary key (id);

create index organizations_app_id_idx
    on organizations (app_id);

create index organizations_status_idx
    on organizations (org_status);

create table org_members
(
    id varchar not null,
    org_id varchar not null
        constraint org_members_organizations_id_fk
            references organizations (id),
    user_info_id varchar not null
        constraint org_members_user_app_info_id_fk
            references user_app_info (id),
    role_id varchar not null
        constraint org_members_hty_roles_hty_role_id_fk
            references hty_roles (hty_role_id),
    member_status varchar not null default 'ACTIVE',
    joined_at timestamp not null default now(),
    created_at timestamp not null default now(),
    created_by varchar
        constraint org_members_hty_users_created_by_fk
            references hty_users (hty_id),
    updated_at timestamp,
    updated_by varchar
        constraint org_members_hty_users_updated_by_fk
            references hty_users (hty_id)
);

create unique index org_members_id_uindex
    on org_members (id);

alter table org_members
    add constraint org_members_pk
        primary key (id);

create unique index org_members_org_user_role_unique_idx
    on org_members (org_id, user_info_id, role_id);

create index org_members_org_id_idx
    on org_members (org_id);

create index org_members_user_info_id_idx
    on org_members (user_info_id);

create index org_members_role_id_idx
    on org_members (role_id);
