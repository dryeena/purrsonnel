//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "profile_field_definitions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub key: String,
    pub label: String,
    pub r#type: String,
    pub validation_json: serde_json::Value,
    pub choices_json: serde_json::Value,
    pub visibility: String,
    pub editable_by: String,
    pub required_for: serde_json::Value,
    pub display_order: i32,
    pub active: bool,
    pub show_on_people_page: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}
impl ActiveModelBehavior for ActiveModel {}
