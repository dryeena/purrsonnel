//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "request_recipients")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub request_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    pub read_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::service_requests::Entity", from = "Column::RequestId", to = "super::service_requests::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Request,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    User,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::service_requests::Entity> for Entity {
    fn to() -> RelationDef { Relation::Request.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
}
