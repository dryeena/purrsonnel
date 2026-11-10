//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub scope_json: serde_json::Value,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
    pub correlation_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::ActorUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    ActorUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::ActorUser.def() }
}
