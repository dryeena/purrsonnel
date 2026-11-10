//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "request_follow_ups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub request_id: Uuid,
    pub reminder_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reminder_processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub escalation_at: Option<chrono::DateTime<chrono::Utc>>,
    pub escalation_user_id: Option<Uuid>,
    pub escalation_processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::service_requests::Entity", from = "Column::RequestId", to = "super::service_requests::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Request,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::EscalationUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    EscalationUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::service_requests::Entity> for Entity {
    fn to() -> RelationDef { Relation::Request.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::EscalationUser.def() }
}
