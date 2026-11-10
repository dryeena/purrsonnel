//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub myfwa_id: Option<String>,
    pub email: Option<String>,
    pub display_name: String,
    pub legal_name: Option<String>,
    pub status: String,
    pub primary_department_id: Option<Uuid>,
    pub personnel_level_id: Option<Uuid>,
    pub volunteer_eligible: bool,
    pub dev_identity: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::PrimaryDepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PrimaryDepartment,
    #[sea_orm(belongs_to = "super::personnel_levels::Entity", from = "Column::PersonnelLevelId", to = "super::personnel_levels::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PersonnelLevel,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::PrimaryDepartment.def() }
}

impl Related<super::personnel_levels::Entity> for Entity {
    fn to() -> RelationDef { Relation::PersonnelLevel.def() }
}
