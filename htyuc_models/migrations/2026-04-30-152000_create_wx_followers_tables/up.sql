CREATE TABLE wx_followers (
    app_id VARCHAR NOT NULL REFERENCES hty_apps(app_id),
    openid VARCHAR NOT NULL,
    refreshed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (app_id, openid)
);

CREATE TABLE wx_follower_infos (
    openid VARCHAR NOT NULL,
    app_id VARCHAR NOT NULL REFERENCES hty_apps(app_id),
    subscribe INT NOT NULL DEFAULT 0,
    unionid VARCHAR NOT NULL DEFAULT '',
    subscribe_time BIGINT NOT NULL DEFAULT 0,
    language VARCHAR NOT NULL DEFAULT '',
    remark VARCHAR NOT NULL DEFAULT '',
    groupid BIGINT NOT NULL DEFAULT 0,
    tagid_list JSONB NOT NULL DEFAULT '[]',
    subscribe_scene VARCHAR NOT NULL DEFAULT '',
    qr_scene BIGINT NOT NULL DEFAULT 0,
    qr_scene_str VARCHAR NOT NULL DEFAULT '',
    refreshed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (openid, app_id)
);

CREATE INDEX idx_wx_follower_infos_unionid ON wx_follower_infos(unionid);
CREATE INDEX idx_wx_follower_infos_app_id_unionid ON wx_follower_infos(app_id, unionid);
