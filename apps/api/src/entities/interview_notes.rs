//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "interview_notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub interview_id: Uuid,
    pub author_user_id: Uuid,
    pub body: String,
    pub visibility: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::interviews::Entity", from = "Column::InterviewId", to = "super::interviews::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Interview,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::AuthorUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    AuthorUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::interviews::Entity> for Entity {
    fn to() -> RelationDef { Relation::Interview.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::AuthorUser.def() }
}
