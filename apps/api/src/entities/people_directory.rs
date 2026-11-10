//! Read-only database view. Never use this entity for writes.
use sea_orm::entity::prelude::*;
#[derive(Clone,Debug,PartialEq,DeriveEntityModel,serde::Serialize,serde::Deserialize)]
#[sea_orm(table_name="people_directory")]
pub struct Model {
 #[sea_orm(primary_key,auto_increment=false)] pub id:Uuid,
 pub myfwa_id:Option<String>,pub display_name:String,pub status:String,
 pub primary_department_id:Option<Uuid>,pub personnel_level_id:Option<Uuid>,pub volunteer_eligible:bool,
 pub created_at:chrono::DateTime<chrono::Utc>,pub updated_at:chrono::DateTime<chrono::Utc>,
 pub level:Option<String>,pub rank:Option<i32>,pub department:Option<String>,
 pub departments:serde_json::Value,pub service_years:serde_json::Value,pub verified_years:i64,pub skills:serde_json::Value,pub profile_fields:serde_json::Value,
}
#[derive(Copy,Clone,Debug,EnumIter,DeriveRelation)]pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
