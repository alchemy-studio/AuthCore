-- Your SQL goes here

-- 首先清理所有孤立的用户关系记录（引用了不存在的用户）
DELETE FROM hty_user_rels r
WHERE NOT EXISTS (SELECT 1 FROM hty_users u WHERE u.hty_id = r.from_user_id)
   OR NOT EXISTS (SELECT 1 FROM hty_users u WHERE u.hty_id = r.to_user_id);

-- 添加外键约束：from_user_id 引用 hty_users
-- ON DELETE CASCADE 表示当用户被删除时，相关的关系记录也会自动删除
ALTER TABLE hty_user_rels
  ADD CONSTRAINT fk_hty_user_rels_from_user
  FOREIGN KEY (from_user_id)
  REFERENCES hty_users(hty_id)
  ON DELETE CASCADE;

-- 添加外键约束：to_user_id 引用 hty_users
ALTER TABLE hty_user_rels
  ADD CONSTRAINT fk_hty_user_rels_to_user
  FOREIGN KEY (to_user_id)
  REFERENCES hty_users(hty_id)
  ON DELETE CASCADE;
