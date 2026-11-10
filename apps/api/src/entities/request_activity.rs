//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "request_activity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub request_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub kind: String,
    pub details: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::service_requests::Entity", from = "Column::RequestId", to = "super::service_requests::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Request,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::ActorUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    ActorUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::service_requests::Entity> for Entity {
    fn to() -> RelationDef { Relation::Request.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::ActorUser.def() }
}
