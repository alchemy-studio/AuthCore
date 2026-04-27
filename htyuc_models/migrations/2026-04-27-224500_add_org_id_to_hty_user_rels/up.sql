ALTER TABLE hty_user_rels
    ADD COLUMN IF NOT EXISTS org_id varchar;

CREATE INDEX IF NOT EXISTS idx_hty_user_rels_org_rel_from_to
    ON hty_user_rels (org_id, rel_type, from_user_id, to_user_id);
