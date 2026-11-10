//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "term_versions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub term_id: Uuid,
    pub version_number: i32,
    pub body_markdown: String,
    pub body_sha256: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub published_by: Option<Uuid>,
    pub superseded_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::terms::Entity", from = "Column::TermId", to = "super::terms::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Term,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::PublishedBy", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    PublishedBy,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::terms::Entity> for Entity {
    fn to() -> RelationDef { Relation::Term.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::PublishedBy.def() }
}
