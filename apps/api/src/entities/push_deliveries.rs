//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "push_deliveries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub notification_id: Uuid,
    pub subscription_id: Uuid,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: chrono::DateTime<chrono::Utc>,
    pub leased_until: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub lease_token: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::notifications::Entity", from = "Column::NotificationId", to = "super::notifications::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Notification,
    #[sea_orm(belongs_to = "super::push_subscriptions::Entity", from = "Column::SubscriptionId", to = "super::push_subscriptions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Subscription,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::notifications::Entity> for Entity {
    fn to() -> RelationDef { Relation::Notification.def() }
}

impl Related<super::push_subscriptions::Entity> for Entity {
    fn to() -> RelationDef { Relation::Subscription.def() }
}
