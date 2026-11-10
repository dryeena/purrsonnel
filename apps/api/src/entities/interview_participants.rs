//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "interview_participants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub interview_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub interviewer_user_id: Uuid,
    pub assigned_by_user_id: Option<Uuid>,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub response: String,
    pub response_at: Option<chrono::DateTime<chrono::Utc>>,
    pub recommendation_notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::interviews::Entity", from = "Column::InterviewId", to = "super::interviews::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Interview,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::InterviewerUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    InterviewerUser,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::AssignedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    AssignedByUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::interviews::Entity> for Entity {
    fn to() -> RelationDef { Relation::Interview.def() }
}
