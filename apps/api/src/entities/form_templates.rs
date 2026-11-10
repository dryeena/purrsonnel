//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "form_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub department_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::DepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Department,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::CreatedBy", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    CreatedBy,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::Department.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::CreatedBy.def() }
}
