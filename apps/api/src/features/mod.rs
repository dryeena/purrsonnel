pub mod people;
pub mod organization;
pub mod terms;
pub mod volunteers;
pub mod hiring;
pub mod lists;
pub mod dashboard;
pub mod configuration;
pub mod lifecycle;
pub mod notifications;
pub mod detail;
pub mod requests;
pub mod push;
pub mod alert_preferences;
pub mod request_follow_up;
pub mod schedules;
use crate::error::Result;
use serde_json::Value;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use uuid::Uuid;
pub async fn audit(tx:&mut DatabaseTransaction,actor:Uuid,action:&str,kind:&str,id:Uuid,before:Value,after:Value)->Result<()> {
 crate::repositories::audit::audit_insert_audit_events(&*tx, (actor).to_owned(), (action).to_owned(), (kind).to_owned(), (id).to_owned(), (before).to_owned(), (after).to_owned()).await?;Ok(())
}
#[derive(serde::Deserialize,Default)]
pub struct Page {pub page:Option<i64>,pub page_size:Option<i64>}
impl Page {pub fn limit(&self)->i64{self.page_size.unwrap_or(25).clamp(1,100)}pub fn offset(&self)->i64{(self.page.unwrap_or(1).clamp(1,100000)-1)*self.limit()}}
