-- 测试数据初始化 SQL

-- 创建 root 应用
INSERT INTO hty_apps (app_id, wx_secret, domain, app_status, pubkey, privkey)
VALUES (
    'root-app-id',
    'root-secret',
    'root',
    'ACTIVE',
    'a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2',
    NULL
) ON CONFLICT (app_id) DO UPDATE SET domain = 'root', app_status = 'ACTIVE';

-- 创建 root 用户
INSERT INTO hty_users (hty_id, union_id, enabled, created_at, real_name)
VALUES (
    'root-hty-id',
    'root-union-id',
    true,
    NOW(),
    'Root User'
) ON CONFLICT (hty_id) DO NOTHING;

-- 创建 root 用户的 app info
INSERT INTO user_app_info (id, hty_id, app_id, is_registered, username, password)
VALUES (
    'root-user-app-info-id',
    'root-hty-id',
    'root-app-id',
    true,
    'root',
    'root'
) ON CONFLICT (id) DO UPDATE SET password = 'root';

-- 创建测试用户（无 SYS_CAN_SUDO 权限）
INSERT INTO hty_users (hty_id, union_id, enabled, created_at, real_name)
VALUES (
    'test-user-hty-id',
    'test-user-union-id',
    true,
    NOW(),
    'Test User'
) ON CONFLICT (hty_id) DO NOTHING;

INSERT INTO user_app_info (id, hty_id, app_id, is_registered, username, password)
VALUES (
    'test-user-app-info-id',
    'test-user-hty-id',
    'root-app-id',
    true,
    'testuser',
    'testpass'
) ON CONFLICT (id) DO UPDATE SET password = 'testpass';

-- 创建 SYS_CAN_SUDO 标签
INSERT INTO hty_tags (tag_id, tag_name, tag_desc)
VALUES (
    'sys-can-sudo-tag-id',
    'SYS_CAN_SUDO',
    'System sudo permission tag'
) ON CONFLICT (tag_id) DO NOTHING;

-- 创建有 sudo 权限的用户
INSERT INTO hty_users (hty_id, union_id, enabled, created_at, real_name)
VALUES (
    'sudo-user-hty-id',
    'sudo-user-union-id',
    true,
    NOW(),
    'Sudo User'
) ON CONFLICT (hty_id) DO NOTHING;

INSERT INTO user_app_info (id, hty_id, app_id, is_registered, username, password)
VALUES (
    'sudo-user-app-info-id',
    'sudo-user-hty-id',
    'root-app-id',
    true,
    'sudouser',
    'sudopass'
) ON CONFLICT (id) DO UPDATE SET password = 'sudopass';

-- 为 sudo 用户分配 SYS_CAN_SUDO 标签
INSERT INTO hty_tag_refs (the_id, hty_tag_id, ref_id, ref_type)
VALUES (
    'sudo-user-tag-ref-id',
    'sys-can-sudo-tag-id',
    'sudo-user-app-info-id',
    'user_app_info'
) ON CONFLICT (the_id) DO NOTHING;
