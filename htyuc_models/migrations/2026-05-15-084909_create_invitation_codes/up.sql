CREATE TABLE invitation_codes (
    id VARCHAR NOT NULL PRIMARY KEY,
    code VARCHAR NOT NULL,
    teacher_id VARCHAR NOT NULL,
    org_id VARCHAR,
    student_user_info_id VARCHAR,
    status VARCHAR NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    consumed_at TIMESTAMP,
    expires_at TIMESTAMP
);

CREATE INDEX idx_invitation_codes_teacher_id ON invitation_codes(teacher_id);
CREATE UNIQUE INDEX idx_invitation_codes_code ON invitation_codes(code);
CREATE INDEX idx_invitation_codes_status ON invitation_codes(status);
