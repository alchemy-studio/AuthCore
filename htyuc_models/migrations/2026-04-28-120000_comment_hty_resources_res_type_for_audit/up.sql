-- hty_resources 本身即为「上传资源」登记处；res_type 用于区分来源（如 org_homepage），便于排查 orphan 与业务表外迁文件。
COMMENT ON COLUMN hty_resources.res_type IS 'Optional provenance tag for uploads (e.g. org_homepage); use with url/created_by for orphan analysis.';
