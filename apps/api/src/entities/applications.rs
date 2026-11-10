//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "applications")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub position_posting_id: Uuid,
    pub applicant_user_id: Uuid,
    pub submitted_myfwa_id: Option<String>,
    pub status: String,
    pub answers_json: serde_json::Value,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decision_actor_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::position_postings::Entity", from = "Column::PositionPostingId", to = "super::position_postings::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PositionPosting,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::ApplicantUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    ApplicantUser,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::DecisionActorId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    DecisionActor,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::position_postings::Entity> for Entity {
    fn to() -> RelationDef { Relation::PositionPosting.def() }
}
