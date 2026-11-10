//! Persistence model; business rules and serialization belong outside this module.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_skills")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub skill_id: Uuid,
    pub proficiency: Option<String>,
    pub notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::users::Entity", from = "Column::UserId", to = "super::users::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    User,
    #[sea_orm(belongs_to = "super::skills::Entity", from = "Column::SkillId", to = "super::skills::Column::Id", on_update = "NoAction", on_delete = "NoAction")]
    Skill,
}
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
}

impl Related<super::skills::Entity> for Entity {
    fn to() -> RelationDef { Relation::Skill.def() }
}
