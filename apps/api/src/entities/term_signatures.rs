//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "term_signatures")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub term_version_id: Uuid,
    pub user_id: Uuid,
    pub typed_name: String,
    pub affirmation_text: String,
    pub consented_at: chrono::DateTime<chrono::Utc>,
    pub server_received_at: chrono::DateTime<chrono::Utc>,
    pub document_sha256: String,
    pub method: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::term_versions::Entity", from = "Column::TermVersionId", to = "super::term_versions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    TermVersion,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    User,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::term_versions::Entity> for Entity {
    fn to() -> RelationDef { Relation::TermVersion.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
}
