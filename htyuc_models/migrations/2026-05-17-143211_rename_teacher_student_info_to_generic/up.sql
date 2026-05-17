-- Rename business-specific columns to generic names
ALTER TABLE user_app_info RENAME COLUMN teacher_info TO extra_info;
ALTER TABLE user_app_info RENAME COLUMN student_info TO extra_info2;
