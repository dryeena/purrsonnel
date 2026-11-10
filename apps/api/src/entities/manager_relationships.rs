//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "manager_relationships")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub manager_user_id: Uuid,
    pub direct_user_id: Uuid,
    pub department_id: Option<Uuid>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::ManagerUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    ManagerUser,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::DirectUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    DirectUser,
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::DepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Department,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::Department.def() }
}
