//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bulk_change_jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub creator_user_id: Uuid,
    pub requested_filter_snapshot_json: serde_json::Value,
    pub target_user_ids_json: serde_json::Value,
    pub operation_json: serde_json::Value,
    pub preview_count: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result_json: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::CreatorUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    CreatorUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::CreatorUser.def() }
}
