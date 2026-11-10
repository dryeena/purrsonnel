//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "volunteer_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub volunteer_user_id: Uuid,
    pub event_edition_id: Option<Uuid>,
    pub department_id: Uuid,
    pub work_date: chrono::NaiveDate,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub minutes_worked: i32,
    pub work_description: String,
    pub status: String,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decided_by_user_id: Option<Uuid>,
    pub decision_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::VolunteerUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    VolunteerUser,
    #[sea_orm(belongs_to = "super::event_editions::Entity", from = "Column::EventEditionId", to = "super::event_editions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    EventEdition,
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::DepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Department,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::DecidedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    DecidedByUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::event_editions::Entity> for Entity {
    fn to() -> RelationDef { Relation::EventEdition.def() }
}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::Department.def() }
}
