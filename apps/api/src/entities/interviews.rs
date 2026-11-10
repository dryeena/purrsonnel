//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "interviews")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub application_id: Uuid,
    pub scheduled_start_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_end_at: chrono::DateTime<chrono::Utc>,
    pub timezone: String,
    pub location_label: String,
    pub location_url: Option<String>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::applications::Entity", from = "Column::ApplicationId", to = "super::applications::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Application,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::CreatedBy", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    CreatedBy,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::applications::Entity> for Entity {
    fn to() -> RelationDef { Relation::Application.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::CreatedBy.def() }
}
