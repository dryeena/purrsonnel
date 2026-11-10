//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "position_postings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub department_id: Uuid,
    pub position_level_id: Uuid,
    pub is_leadership: bool,
    pub description_markdown: String,
    pub status: String,
    pub created_by_user_id: Uuid,
    pub hiring_owner_user_id: Uuid,
    pub form_template_version_id: Uuid,
    pub opened_at: Option<chrono::DateTime<chrono::Utc>>,
    pub closes_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::departments::Entity", from = "Column::DepartmentId", to = "super::departments::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Department,
    #[sea_orm(belongs_to = "super::position_levels::Entity", from = "Column::PositionLevelId", to = "super::position_levels::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PositionLevel,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::CreatedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    CreatedByUser,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::HiringOwnerUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    HiringOwnerUser,
    #[sea_orm(belongs_to = "super::form_template_versions::Entity", from = "Column::FormTemplateVersionId", to = "super::form_template_versions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    FormTemplateVersion,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::departments::Entity> for Entity {
    fn to() -> RelationDef { Relation::Department.def() }
}

impl Related<super::position_levels::Entity> for Entity {
    fn to() -> RelationDef { Relation::PositionLevel.def() }
}

impl Related<super::form_template_versions::Entity> for Entity {
    fn to() -> RelationDef { Relation::FormTemplateVersion.def() }
}
