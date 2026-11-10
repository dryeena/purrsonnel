//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "position_levels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub label: String,
    pub personnel_level_id: Option<Uuid>,
    pub active: bool,
    pub sort_order: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::personnel_levels::Entity", from = "Column::PersonnelLevelId", to = "super::personnel_levels::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PersonnelLevel,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::personnel_levels::Entity> for Entity {
    fn to() -> RelationDef { Relation::PersonnelLevel.def() }
}
