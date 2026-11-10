//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "profile_field_values")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub definition_id: Uuid,
    pub value_json: serde_json::Value,
    pub updated_by_user_id: Option<Uuid>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    User,
    #[sea_orm(belongs_to = "super::profile_field_definitions::Entity", from = "Column::DefinitionId", to = "super::profile_field_definitions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Definition,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UpdatedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    UpdatedByUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::profile_field_definitions::Entity> for Entity {
    fn to() -> RelationDef { Relation::Definition.def() }
}
