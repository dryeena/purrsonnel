use sea_orm::{ColumnTrait,ConnectionTrait,DbErr,EntityTrait,QueryFilter,QuerySelect};
use uuid::Uuid;
use crate::entities::users;

/// Development identities must be active and explicitly marked fictional.
pub async fn resolve_select_users<C:ConnectionTrait>(db:&C,subject:String)->Result<Uuid,DbErr>{
 users::Entity::find().select_only().column(users::Column::Id)
  .filter(users::Column::Email.eq(subject)).filter(users::Column::DevIdentity.eq(true))
  .filter(users::Column::Status.eq("active")).into_tuple::<Uuid>().one(db).await?
  .ok_or_else(||DbErr::RecordNotFound("Development identity not found".into()))
}
