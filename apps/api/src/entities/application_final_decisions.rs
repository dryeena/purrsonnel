//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "application_final_decisions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub application_id: Uuid,
    pub decision: String,
    pub decided_by_user_id: Uuid,
    pub reason: String,
    pub decided_at: chrono::DateTime<chrono::Utc>,
    pub idempotency_key: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::applications::Entity", from = "Column::ApplicationId", to = "super::applications::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Application,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::DecidedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    DecidedByUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::applications::Entity> for Entity {
    fn to() -> RelationDef { Relation::Application.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::DecidedByUser.def() }
}
