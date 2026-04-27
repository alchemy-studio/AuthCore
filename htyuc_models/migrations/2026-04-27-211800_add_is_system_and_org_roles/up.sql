alter table hty_roles
    add column if not exists is_system boolean not null default false;

create table if not exists org_roles
(
    id varchar not null,
    org_id varchar not null
        constraint org_roles_organizations_id_fk
            references organizations (id),
    role_id varchar not null
        constraint org_roles_hty_roles_id_fk
            references hty_roles (hty_role_id),
    role_status varchar not null default 'ACTIVE',
    created_at timestamp not null default now(),
    created_by varchar
        constraint org_roles_hty_users_created_by_fk
            references hty_users (hty_id),
    updated_at timestamp,
    updated_by varchar
        constraint org_roles_hty_users_updated_by_fk
            references hty_users (hty_id)
);

create unique index if not exists idx_org_roles_org_role_unique
    on org_roles (org_id, role_id);

create index if not exists idx_org_roles_org_id
    on org_roles (org_id);

create index if not exists idx_org_roles_role_id
    on org_roles (role_id);
