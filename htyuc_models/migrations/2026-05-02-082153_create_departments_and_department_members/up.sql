create table departments
(
    id varchar not null,
    org_id varchar not null
        constraint departments_organizations_id_fk
            references organizations (id),
    dept_name varchar not null,
    dept_desc varchar,
    supervisor_user_info_id varchar
        constraint departments_user_app_info_id_fk
            references user_app_info (id),
    is_default boolean not null default false,
    dept_status varchar not null default 'ACTIVE',
    created_at timestamp not null default now(),
    created_by varchar
        constraint departments_hty_users_created_by_fk
            references hty_users (hty_id),
    updated_at timestamp,
    updated_by varchar
        constraint departments_hty_users_updated_by_fk
            references hty_users (hty_id),
    is_delete boolean not null default false
);

create unique index departments_id_uindex
    on departments (id);

alter table departments
    add constraint departments_pk
        primary key (id);

-- only one default department per org
create unique index departments_org_default_unique_idx
    on departments (org_id) where is_default = true and is_delete = false;

create index departments_org_id_idx
    on departments (org_id);

create index departments_supervisor_idx
    on departments (supervisor_user_info_id);

create table department_members
(
    id varchar not null,
    department_id varchar not null
        constraint department_members_departments_id_fk
            references departments (id),
    org_id varchar not null
        constraint department_members_organizations_id_fk
            references organizations (id),
    user_info_id varchar not null
        constraint department_members_user_app_info_id_fk
            references user_app_info (id),
    member_status varchar not null default 'ACTIVE',
    joined_at timestamp not null default now(),
    created_at timestamp not null default now(),
    created_by varchar
        constraint department_members_hty_users_created_by_fk
            references hty_users (hty_id),
    updated_at timestamp,
    updated_by varchar
        constraint department_members_hty_users_updated_by_fk
            references hty_users (hty_id)
);

create unique index department_members_id_uindex
    on department_members (id);

alter table department_members
    add constraint department_members_pk
        primary key (id);

-- one active membership per teacher per department
create unique index department_members_dept_user_unique_idx
    on department_members (department_id, user_info_id) where member_status = 'ACTIVE';

create index department_members_dept_id_idx
    on department_members (department_id);

create index department_members_org_id_idx
    on department_members (org_id);

create index department_members_user_info_id_idx
    on department_members (user_info_id);
