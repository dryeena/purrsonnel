//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "volunteer_adjustments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub entry_id: Uuid,
    pub actor_user_id: Uuid,
    pub reason: String,
    pub previous_json: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::volunteer_entries::Entity", from = "Column::EntryId", to = "super::volunteer_entries::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Entry,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::ActorUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    ActorUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::volunteer_entries::Entity> for Entity {
    fn to() -> RelationDef { Relation::Entry.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::ActorUser.def() }
}
