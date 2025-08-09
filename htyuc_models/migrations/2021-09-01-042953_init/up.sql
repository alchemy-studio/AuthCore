-- Your SQL goes here

--
-- Name: hty_apps; Type: TABLE; Schema: public; Owner: htyuc
--

CREATE TABLE hty_apps
(
    app_id character varying NOT NULL,
    secret character varying NOT NULL,
    domain character varying
);


CREATE TABLE hty_resources
(
    filename        character varying,
    app_id          character varying NOT NULL,
    hty_resource_id character varying NOT NULL,
    created_at      timestamp without time zone,
    url             character varying NOT NULL,
    res_type        integer           NOT NULL,
    created_by      character varying
);


COMMENT ON TABLE hty_resources IS '保存各app上传资源的源信息';



CREATE TABLE hty_users
(
    hty_id     character varying    NOT NULL,
    union_id   character varying,
    enabled    boolean DEFAULT true NOT NULL,
    created_at timestamp without time zone,
    real_name  character varying
);


CREATE TABLE hty_visitors
(
    hty_id         integer,
    id             character varying           NOT NULL,
    meta           jsonb DEFAULT '{}'::jsonb   NOT NULL,
    last_logged_at timestamp without time zone NOT NULL
);



CREATE TABLE user_app_info
(
    hty_id        character varying NOT NULL,
    app_id        character varying,
    openid        character varying,
    is_registered boolean           NOT NULL,
    id            character varying NOT NULL,
    user_role     integer,
    username      character varying
);


ALTER TABLE ONLY hty_resources
    ADD CONSTRAINT hty_resources_pk PRIMARY KEY (hty_resource_id);


ALTER TABLE ONLY hty_users
    ADD CONSTRAINT hty_users_pk PRIMARY KEY (hty_id);



ALTER TABLE ONLY hty_visitors
    ADD CONSTRAINT hty_visitors_pk PRIMARY KEY (id);


ALTER TABLE ONLY user_app_info
    ADD CONSTRAINT user_app_info_pk PRIMARY KEY (id);


ALTER TABLE ONLY hty_apps
    ADD CONSTRAINT hty_apps_pkey PRIMARY KEY (app_id);



CREATE UNIQUE INDEX hty_resources_id_uindex ON hty_resources USING btree (hty_resource_id);



CREATE UNIQUE INDEX hty_users_union_id_uindex ON hty_users USING btree (union_id);


CREATE UNIQUE INDEX hty_visitors_id_uindex ON hty_visitors USING btree (id);


CREATE UNIQUE INDEX user_app_info_id_uindex ON user_app_info USING btree (id);


CREATE UNIQUE INDEX hty_apps_app_id_uindex ON hty_apps USING btree (app_id);


CREATE UNIQUE INDEX hty_apps_domain_uindex ON hty_apps USING btree (domain);


ALTER TABLE ONLY hty_resources
    ADD CONSTRAINT hty_resources_hty_apps_app_id_fk FOREIGN KEY (app_id) REFERENCES hty_apps (app_id);



ALTER TABLE ONLY hty_resources
    ADD CONSTRAINT hty_resources_hty_users_hty_id_fk FOREIGN KEY (created_by) REFERENCES hty_users (hty_id);



ALTER TABLE ONLY user_app_info
    ADD CONSTRAINT user_app_info_hty_apps_app_id_fk FOREIGN KEY (app_id) REFERENCES hty_apps (app_id);


ALTER TABLE ONLY user_app_info
    ADD CONSTRAINT user_app_info_hty_users_hty_id_fk FOREIGN KEY (hty_id) REFERENCES hty_users (hty_id);


--
-- PostgreSQL database dump complete
--

