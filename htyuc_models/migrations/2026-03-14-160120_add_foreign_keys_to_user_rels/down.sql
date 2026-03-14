-- This file should undo anything in `up.sql`

-- 删除外键约束
ALTER TABLE hty_user_rels DROP CONSTRAINT IF EXISTS fk_hty_user_rels_from_user;
ALTER TABLE hty_user_rels DROP CONSTRAINT IF EXISTS fk_hty_user_rels_to_user;
