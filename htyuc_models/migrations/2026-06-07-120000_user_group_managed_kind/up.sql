ALTER TABLE hty_user_group
    ADD COLUMN IF NOT EXISTS managed_kind VARCHAR NOT NULL DEFAULT 'MANUAL',
    ADD COLUMN IF NOT EXISTS managed_ref_id VARCHAR NULL;

CREATE INDEX IF NOT EXISTS idx_hty_user_group_managed_kind ON hty_user_group (managed_kind);
