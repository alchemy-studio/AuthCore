drop index if exists idx_org_roles_role_id;
drop index if exists idx_org_roles_org_id;
drop index if exists idx_org_roles_org_role_unique;
drop table if exists org_roles;

alter table hty_roles
    drop column if exists is_system;
