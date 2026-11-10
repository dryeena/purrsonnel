use sea_orm::{ActiveModelTrait,ConnectionTrait,DbErr,Set};
use serde_json::Value;
use uuid::Uuid;
use crate::entities::audit_events;

/// Evidence is append-only; the database supplies its ID, correlation and timestamp.
pub async fn audit_insert_audit_events<C:ConnectionTrait>(db:&C,actor:Uuid,action:String,kind:String,id:Uuid,before:Value,after:Value)->Result<(),DbErr>{
 audit_events::ActiveModel{
  actor_user_id:Set(Some(actor)),action:Set(action),resource_type:Set(kind),resource_id:Set(Some(id)),
  before_json:Set(Some(before)),after_json:Set(Some(after)),..Default::default()
 }.insert(db).await?;
 Ok(())
}
