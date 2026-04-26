-- 回滚：恢复旧占位符命名
DO $$
DECLARE
  rec RECORD;
  inner_text TEXT;
  new_inner TEXT;
BEGIN
  FOR rec IN
    SELECT td.id, t.template_key, td.template_text->>'val' AS inner_json
    FROM hty_template_data td
    JOIN hty_template t ON t.id = td.template_id
    WHERE td.app_id = '03f9a505-48f6-4e18-b1f7-fa763ce63a6e'
      AND t.template_key IN ('create_lianxi','delete_lianxi','create_piyue',
                             'create_resource_note_group','teacher_comment_piyue',
                             'student_comment_piyue')
  LOOP
    inner_text := rec.inner_json;

    IF inner_text LIKE '%COURSE_NAME%' OR inner_text LIKE '%COURSE_SECTION_NAME%' THEN
      new_inner := inner_text;

      -- pass 1: 中文 课程：→ 曲目：（独立中文，无冲突）
      new_inner := replace(new_inner, '课程：', '曲目：');

      -- pass 2: COURSE_SECTION_NAME → QUMU_SECTION_NAME（最长优先，避免子串冲突）
      new_inner := replace(new_inner, 'COURSE_SECTION_NAME', 'QUMU_SECTION_NAME');

      -- pass 3: COURSE_NAME → QUMU_NAME
      new_inner := replace(new_inner, 'COURSE_NAME', 'QUMU_NAME');

      UPDATE hty_template_data
      SET template_text = jsonb_set(template_text, '{val}', to_jsonb(new_inner), false)
      WHERE id = rec.id;
    END IF;
  END LOOP;
END $$;
