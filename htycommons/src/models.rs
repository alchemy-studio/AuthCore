use std::fmt::Debug;
use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::db::CommonTask;
use crate::{impl_jsonb_boilerplate, impl_typed_jsonb_boilerplate};
use diesel::sql_types::Jsonb;
use diesel::pg::PgValue;
use diesel::pg::Pg;
// use diesel::helper_types::IsNull;
use std::io::Write;

#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct PushInfo {
    // deprecated. to be removed.
    // 可以拿来保存类似于加点数据里的comments
    pub comment: Option<ReqComment>,
    pub comment_id: Option<String>,
    pub comment_msg: Option<String>,
    pub comment_time: Option<String>,
    pub daka_id: Option<String>,
    pub end_by: Option<String>,
    pub first: Option<String>, // in wx template
    pub hty_id2: Option<String>,
    pub hty_id: Option<String>,
    pub jihua_id: Option<String>,
    pub kecheng_id: Option<String>,
    pub kecheng_name: Option<String>,
    pub lianxi_id: Option<String>,
    pub notify_type: Option<String>,
    pub piyue_id: Option<String>,
    pub qumu_name: Option<String>,
    pub qumu_section_name: Option<String>,
    pub ref_id: Option<String>,
    pub ref_type: Option<String>,
    pub reject_reason: Option<String>,
    pub remark: Option<String>,
    pub resource_note_group_id: Option<String>,
    pub serial: Option<String>,
    pub start_from: Option<String>,
    pub student_name: Option<String>, // list合并后的所有学生姓名，逗号分隔，发给小程序接口发通知用。
    pub teacher_name: Option<String>, // list合并后的所有老师姓名，逗号分隔，发给小程序接口发通知用。
    pub to_role_id: Option<String>,
    pub from_app_id: Option<String>,
}

impl_jsonb_boilerplate!(PushInfo);


// this need to be here because `PushInfo` used it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReqComment {
    pub id: Option<String>,
    pub ref_id: Option<String>,
    pub ref_type: Option<String>,
    // currently diesel doesn't support self reference.
    pub parent_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub creator_name: Option<String>,
    pub content: Option<CommentContent>,
    pub is_delete: Option<bool>,
    pub comment_type: Option<String>,
    pub comment_status: Option<String>,
    pub has_piyue: Option<bool>,
    pub has_score: Option<bool>,
    pub ref_resources: Option<Vec<ReqRefResource>>,
    // this is selected out by runtime, not stored in db.
    pub creator_role_key: Option<String>,
}


#[derive(AsExpression, FromSqlRow, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct CommentContent {
    pub text: Option<String>,
    pub meta: Option<String>,
    // deprecated
    pub audio_id: Option<String>,
    pub audio_url: Option<String>,
    pub picture_id: Option<String>,
    pub picture_url: Option<String>,
    pub video_id: Option<String>,
    pub video_url: Option<String>,
    pub task: Option<ConvertAudioFileTask>,
    pub photos: Option<Vec<ReqHtyResource>>,
    // end of deprecated
}

impl_jsonb_boilerplate!(CommentContent);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReqHtyResource {
    pub app_id: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub created_by: Option<String>,
    pub filename: Option<String>,
    pub hty_resource_id: Option<String>,
    pub res_type: Option<String>,
    pub url: Option<String>,
    pub tasks: Option<MultiVals<CommonTask>>,
    pub compress_processed: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
    pub updated_by: Option<String>,
}

#[derive(AsExpression, FromSqlRow, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct ConvertAudioFileTask {
    pub media_id: Option<String>,
    pub task_id: Option<String>,
    pub task_type: String,
}

impl Default for ConvertAudioFileTask {
    fn default() -> ConvertAudioFileTask {
        ConvertAudioFileTask {
            media_id: None,
            task_id: None,
            task_type: "CONVERT_AUDIO_FILE".to_string(),
        }
    }
}

impl_jsonb_boilerplate!(ConvertAudioFileTask);


// https://serde.rs/lifetimes.html
// https://stackoverflow.com/questions/61473323/cannot-infer-type-for-type-parameter-when-deriving-deserialize-for-a-type-with-a
#[derive(AsExpression, FromSqlRow, Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
#[serde(bound = "")]
pub struct MultiVals<T: Debug + Serialize + DeserializeOwned> {
    pub vals: Option<Vec<T>>,
}

impl_typed_jsonb_boilerplate!(MultiVals);

#[derive(serde_derive::Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReqRefResource {
    pub id: Option<String>,
    pub hty_resource_id: Option<String>,
    pub ref_id: Option<String>,
    pub ref_type: Option<String>,
    pub resource_url: Option<String>,
    pub resource_type: Option<String>,
    pub ref_name: Option<String>,
    pub ref_desc: Option<String>,
    // pub meta: Option<MultiVals<ReqRefResource>>, // deprecated, re-design
    pub tasks: Option<MultiVals<CommonTask>>,
    pub is_shifan: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub compress_processed: Option<bool>,
    pub created_by: Option<String>,
    pub synced_with_hty_resource: Option<bool>,
    pub updated_by: Option<String>,
}


#[derive(AsExpression, FromSqlRow, Debug, Default, serde_derive::Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct CommonTongzhiContent {
    pub to_user: Option<String>,
    pub from_user: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub content: Option<String>,
    pub qumu_sections: Option<Vec<String>>,
    pub piyue_id: Option<String>,
    pub beizhu: Option<String>,
    pub lianxi_id: Option<String>,
    pub jihua_id: Option<String>,
    pub jihua_start_from: Option<NaiveDateTime>,
    pub jihua_end_at: Option<NaiveDateTime>,
    pub daka_id: Option<String>,
    pub daka_start_date: Option<NaiveDateTime>,
    pub daka_duration_days: Option<i32>,
}

impl_jsonb_boilerplate!(CommonTongzhiContent);

#[derive(AsExpression, FromSqlRow, Debug, Default, serde_derive::Serialize, Deserialize, PartialEq, Clone)]
#[diesel(sql_type = Jsonb)]
pub struct TongzhiMeta {
    pub val: Option<String>,
}

impl_jsonb_boilerplate!(TongzhiMeta);