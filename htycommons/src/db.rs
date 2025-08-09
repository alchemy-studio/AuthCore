use crate::common::{HtyErr, HtyErrCode};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::ops::{Deref};
use std::sync::Arc;
use anyhow::anyhow;
use axum::extract::{FromRef, FromRequestParts};
use diesel::pg::{Pg, PgValue};
// use diesel::serialize::IsNull;
use std::io::Write;
use axum::http::request::Parts;
use axum::http::StatusCode;
use diesel::sql_types::Jsonb;
use diesel::{Connection, PgConnection};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
// use crate::{impl_jsonb_boilerplate, impl_typed_jsonb_boilerplate};
use crate::web::internal_error;
use crate::{impl_jsonb_boilerplate, impl_typed_jsonb_boilerplate};

pub type PgPool = Pool<PgConnMgr>;

pub type PgConnMgr = ConnectionManager<PgConnection>;

pub type PooledPgConn = PooledConnection<PgConnMgr>;

// params: hty_id, app_key, etc.
// trait RWT<T, U> = Fn(Option<HashMap<String, U>>, &PgConnection) -> anyhow::Result<T>;

pub type ReadWriteTask<T, U> = dyn Fn(Option<HashMap<String, U>>, &mut PgConnection) -> anyhow::Result<T>;

pub struct DbConn(pub PooledPgConn);

pub static READ: &'static str = "Read";
pub static UNREAD: &'static str = "Unread";

pub fn pool(db_url: &str) -> PgPool {
    info!("全局数据库连接池初始化📦");

    dotenv().ok();

    let manager = PgConnMgr::new(db_url);
    let max_size = env::var("POOL_SIZE")
        .expect("POOL_SIZE must be set")
        .parse::<u32>()
        .unwrap();

    Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("数据库连接池创建失败！")
}

pub fn get_conn(pool: &PgPool) -> PooledPgConn {
    match pool.get() {
        Ok(conn) => conn,
        Err(_) => panic!("数据库连接失败！"),
    }
}

pub struct DbState {
    pub pool: PgPool,
}

type MyDbState = Arc<DbState>;


impl<S> FromRequestParts<S> for DbConn
    where
        MyDbState: FromRef<S>,
        S: Send + Sync
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db_pool = MyDbState::from_ref(state);
        let conn = db_pool.pool.get().map_err(internal_error)?;

        Ok(Self(conn))
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn extract_conn(conn: DbConn) -> PooledPgConn {
    conn.0
}

pub fn fetch_db_conn(db_pool: &Arc<DbState>) -> anyhow::Result<DbConn> {
    Ok(DbConn(db_pool.pool.get()?))
}

pub fn exec_read_write_task<T, U>(
    f: Box<ReadWriteTask<T, U>>,
    params: Option<HashMap<String, U>>,
    conn: &mut PgConnection,
) -> anyhow::Result<T> {
    match conn.transaction::<_, diesel::result::Error, _>(move |conn| Ok(f(params, conn)))
    {
        Ok(ok) => match ok {
            Ok(ok) => Ok(ok),
            Err(e) => Err(anyhow!(HtyErr {
                code: HtyErrCode::DbErr,
                reason: Some(format!("{:?}", e)),
            })),
        },
        Err(e) => Err(anyhow!(HtyErr {
            code: HtyErrCode::InternalErr,
            reason: Some(format!("{:?}", e)),
        })),
    }
}


pub fn get_uc_db_url() -> String {
    env::var("UC_DB_URL").expect("UC_DB_URL not set!")
}

pub fn set_uc_db_url(url: &String) {
    env::set_var("UC_DB_URL", url);
}

pub fn get_ws_db_url() -> String {
    env::var("WS_DB_URL").expect("WS_DB_URL not set!")
}

pub fn set_ws_db_url(url: &String) {
    env::set_var("WS_DB_URL", url);
}

pub fn get_kc_db_url() -> String {
    env::var("KC_DB_URL").expect("KC_DB_URL not set!")
}

pub fn set_kc_db_url(url: &String) {
    env::set_var("KC_DB_URL", url);
}


// --------

// #[derive(AsExpression, FromSqlRow, Debug, Serialize, Deserialize, PartialEq, Clone)]
// #[sql_type = "Jsonb"]
// pub struct CommonTasks {
//     pub tasks: Option<Vec<CommonTask>>,
// }
// impl_jsonb_boilerplate!(CommonTasks);


#[derive(AsExpression, FromSqlRow, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct CommonTask {
    pub task_id: Option<String>,
    pub task_type: Option<String>,
    pub task_from: Option<String>,
    pub task_status: Option<String>,
    pub task_result: Option<HashMap<String, String>>,
    pub duration: Option<f64>,
    // 这里只存result相关数据
    //保存一些非结构化数据
    pub task_meta: Option<TaskMeta>,
}


impl_jsonb_boilerplate!(CommonTask);


#[derive(AsExpression, FromSqlRow, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct TaskMeta {
    pub media_id: Option<String>,
    pub data: Option<HashMap<String, String>>,
    // 这里可以放所有相关数据
    pub err: Option<String>,
}

impl_jsonb_boilerplate!(TaskMeta);

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct CommonMeta {
    pub meta: Option<HashMap<String, String>>,
}

impl_jsonb_boilerplate!(CommonMeta);


#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
#[serde(bound = "")]
pub struct SingleVal<T: Debug + Serialize + DeserializeOwned + Clone> {
    pub val: Option<T>,
}

impl_typed_jsonb_boilerplate!(SingleVal);

#[macro_export]
macro_rules! impl_jsonb_boilerplate {
    ($name: ident) => {
        impl ::diesel::deserialize::FromSql<::diesel::sql_types::Jsonb, ::diesel::pg::Pg>
            for $name
        {
            fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
                let value = <::serde_json::Value as ::diesel::deserialize::FromSql<
                    ::diesel::sql_types::Jsonb,
                    ::diesel::pg::Pg,
                >>::from_sql(bytes)?;
                Ok(::serde_json::from_value(value)?)
            }
        }

        impl ::diesel::serialize::ToSql<::diesel::sql_types::Jsonb, Pg> for $name {
            fn to_sql<'b>(
                &'b self,
                out: &mut ::diesel::serialize::Output<'b, '_, Pg>,
            ) -> ::diesel::serialize::Result {
                out.write_all(&[1])?;
                ::serde_json::to_writer(out, &::serde_json::to_value(self)?)
                    .map(|_| diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_typed_jsonb_boilerplate {
    ($name: ident) => {
        impl <T: Debug + Serialize + DeserializeOwned + Clone> ::diesel::deserialize::FromSql<::diesel::sql_types::Jsonb, ::diesel::pg::Pg>
            for $name<T>
        {
            fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
                let value = <::serde_json::Value as ::diesel::deserialize::FromSql<
                    ::diesel::sql_types::Jsonb,
                    ::diesel::pg::Pg,
                >>::from_sql(bytes)?;
                Ok(::serde_json::from_value(value)?)
            }
        }

        impl<T> ::diesel::serialize::ToSql<::diesel::sql_types::Jsonb, Pg> for $name<T>
            where
                T: Debug + Serialize + DeserializeOwned + Clone,
        {
            fn to_sql<'b>(
                &'b self,
                out: &mut ::diesel::serialize::Output<'b, '_, Pg>,
            ) -> ::diesel::serialize::Result {
                out.write_all(&[1])?;
                ::serde_json::to_writer(out, &::serde_json::to_value(self)?)
                    .map(|_| diesel::serialize::IsNull::No)
                    .map_err(Into::into)
            }
        }
    };
}