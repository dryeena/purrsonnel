//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "permission_grants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub granted_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    User,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::GrantedBy", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    GrantedBy,
}
impl ActiveModelBehavior for ActiveModel {}
