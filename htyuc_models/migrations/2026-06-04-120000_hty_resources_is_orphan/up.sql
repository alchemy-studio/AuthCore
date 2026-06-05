ALTER TABLE hty_resources
    ADD COLUMN is_orphan BOOLEAN NOT NULL DEFAULT true;

COMMENT ON COLUMN hty_resources.is_orphan IS
    'true: 已上传但未挂业务（ref_resources/course_material 等）；业务保存认领后置 false。';

CREATE INDEX idx_hty_resources_orphan_created
    ON hty_resources (is_orphan, created_at)
    WHERE is_orphan = true;

-- 存量：无法跨库关联 htyws.ref_resources，一律视为已认领，避免误清理
UPDATE hty_resources SET is_orphan = false;
