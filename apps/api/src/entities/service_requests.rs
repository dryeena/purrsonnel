//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "service_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub sender_user_id: Uuid,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_department_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub urgency: String,
    pub due_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub assigned_to_user_id: Option<Uuid>,
    pub idempotency_key: Uuid,
    pub payload_sha256: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_reminded_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::SenderUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    SenderUser,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::RecipientUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    RecipientUser,
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::RecipientDepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    RecipientDepartment,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::AssignedToUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    AssignedToUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::RecipientDepartment.def() }
}
