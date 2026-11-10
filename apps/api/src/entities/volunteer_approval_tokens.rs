//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "volunteer_approval_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub entry_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub redeemed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub redeemed_by_user_id: Option<Uuid>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::volunteer_entries::Entity", from = "Column::EntryId", to = "super::volunteer_entries::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Entry,
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::RedeemedByUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    RedeemedByUser,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::volunteer_entries::Entity> for Entity {
    fn to() -> RelationDef { Relation::Entry.def() }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::RedeemedByUser.def() }
}
