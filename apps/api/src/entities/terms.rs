//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "terms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub audience: String,
    pub minimum_personnel_rank: Option<i32>,
    pub requires_volunteer: Option<bool>,
    pub active: bool,
    pub effective_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::CreatedBy", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    CreatedBy,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::CreatedBy.def() }
}
