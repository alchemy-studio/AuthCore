DROP INDEX IF EXISTS idx_hty_user_rels_org_rel_from_to;

ALTER TABLE hty_user_rels
    DROP COLUMN IF EXISTS org_id;
