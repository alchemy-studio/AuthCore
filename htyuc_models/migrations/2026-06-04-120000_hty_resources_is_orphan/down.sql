DROP INDEX IF EXISTS idx_hty_resources_orphan_created;
ALTER TABLE hty_resources DROP COLUMN IF EXISTS is_orphan;
