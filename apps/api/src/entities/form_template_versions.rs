//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "form_template_versions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub template_id: Uuid,
    pub version_number: i32,
    pub schema_json: serde_json::Value,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::form_templates::Entity", from = "Column::TemplateId", to = "super::form_templates::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Template,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::form_templates::Entity> for Entity {
    fn to() -> RelationDef { Relation::Template.def() }
}
