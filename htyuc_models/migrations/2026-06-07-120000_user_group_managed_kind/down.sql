ALTER TABLE hty_user_group
    DROP COLUMN IF EXISTS managed_ref_id,
    DROP COLUMN IF EXISTS managed_kind;
