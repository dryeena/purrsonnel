//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "departments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub division_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub lead_user_id: Option<Uuid>,
    pub description: String,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::LeadUserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    LeadUser,
    #[sea_orm(belongs_to = "super::divisions::Entity", from = "Column::DivisionId", to = "super::divisions::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Division,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::LeadUser.def() }
}

impl Related<super::divisions::Entity> for Entity {
    fn to() -> RelationDef { Relation::Division.def() }
}
