-- 更新模版文本占位符命名：QUMU_NAME → COURSE_NAME, QUMU_SECTION_NAME → COURSE_SECTION_NAME
-- 更新中文：曲目：→ 课程：
-- 仅更新炼金工坊公众号 (app_id = '03f9a505-48f6-4e18-b1f7-fa763ce63a6e')
-- template_text 是 SingleVal 结构：{"val": "<inner_json_string>"}，需先提取 val 再操作

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

    IF inner_text LIKE '%QUMU%' OR inner_text LIKE '%曲目%' THEN
      new_inner := inner_text;

      -- pass 1: QUMU_SECTION_NAME → COURSE_SECTION_NAME（必须先做，QUMU_NAME 是其子串）
      new_inner := replace(new_inner, 'QUMU_SECTION_NAME', 'COURSE_SECTION_NAME');

      -- pass 2: QUMU_NAME → COURSE_NAME
      new_inner := replace(new_inner, 'QUMU_NAME', 'COURSE_NAME');

      -- pass 3: 中文 曲目：→ 课程：
      new_inner := replace(new_inner, '曲目：', '课程：');

      -- pass 4: 独立的 SECTION_NAME → COURSE_SECTION_NAME（前面必须是标点/空格/括号）
      new_inner := regexp_replace(new_inner, '([:： (（])SECTION_NAME', '\1COURSE_SECTION_NAME', 'g');

      UPDATE hty_template_data
      SET template_text = jsonb_set(template_text, '{val}', to_jsonb(new_inner), false)
      WHERE id = rec.id;
    END IF;
  END LOOP;
END $$;
