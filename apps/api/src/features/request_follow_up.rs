//! Follow-through uses the same database transaction as its inbox/push evidence.
//! The scheduler runs even when Web Push is not configured, and never broadcasts
//! to a new department or escalates to an inferred manager.
use axum::{extract::{Path,State},Json};
use chrono::{DateTime,Duration,Utc};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::{DatabaseTransaction,TransactionTrait};
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result}};

pub async fn activity(tx:&mut DatabaseTransaction,id:Uuid,actor:Option<Uuid>,kind:&str,details:Value)->Result<()> {
 crate::repositories::request_follow_up::activity_insert_request_activity(&*tx, (id).to_owned(), (actor).to_owned(), (kind).to_owned(), (details).to_owned()).await?;Ok(())
}
// Returns queued devices and policy-suppressed devices. A queued alert may be deferred.
pub async fn notify(tx:&mut DatabaseTransaction,push_enabled:bool,user:Uuid,id:Uuid,kind:&str,urgency:&str)->Result<(u64,u64)> {
 let notification:Option<Uuid>=crate::repositories::request_follow_up::notify_insert_notifications(&*tx, (user).to_owned(), (kind).to_owned(), (json!({"request_id":id,"url":format!("/requests?id={id}")})).to_owned(), (id).to_owned()).await?;
 let Some(notification)=notification else{return Ok((0,0));};
 if !push_enabled{return Ok((0,0));}
 let rows=crate::repositories::request_follow_up::notify_insert_push_deliveries(&*tx, (notification).to_owned(), (user).to_owned(), (urgency).to_owned()).await?;
 let suppressed=rows.iter().try_fold(0u64,|count,r|Ok::<_,sea_orm::DbErr>(count+u64::from(r.try_get::<String>("","status")?=="suppressed")))?;
 Ok((rows.len() as u64-suppressed,suppressed))
}
pub async fn details(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>> {
 let sender:Uuid=crate::repositories::request_follow_up::details_select_service_requests(&s.db, (id).to_owned(), (p.id).to_owned()).await?;
 let follow_up:Option<Value>=crate::repositories::request_follow_up::details_select_request_follow_ups(&s.db, (id).to_owned()).await?;
 let timeline:Vec<Value>=crate::repositories::request_follow_up::details_select_request_activity(&s.db, (id).to_owned()).await?;
 let delivery=if sender==p.id {Some(crate::repositories::request_follow_up::details_select_push_deliveries::<_,Value>(&s.db, (id).to_owned()).await?)}else{None};
 Ok(Json(json!({"follow_up":follow_up,"activity":timeline,"delivery":delivery})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct Schedule {pub reminder_at:Option<DateTime<Utc>>,pub escalation_at:Option<DateTime<Utc>>,pub escalation_user_id:Option<Uuid>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
pub struct ScheduleInput {
 pub reminder_at:Option<DateTime<Utc>>,pub escalation_at:Option<DateTime<Utc>>,pub escalation_user_id:Option<Uuid>,
 pub reminder_after_minutes:Option<i64>,pub escalation_after_minutes:Option<i64>,
}
pub fn resolve_schedule(b:ScheduleInput,now:DateTime<Utc>)->Result<Schedule> {
 fn resolve(at:Option<DateTime<Utc>>,after:Option<i64>,now:DateTime<Utc>)->Result<Option<DateTime<Utc>>>{
  if at.is_some()&&after.is_some(){return Err(ApiError::bad("Choose an absolute time or relative minutes, not both"));}
  if let Some(minutes)=after{if !(15..=43200).contains(&minutes){return Err(ApiError::bad("Choose follow-up delays between 15 minutes and 30 days"));}return Ok(Some(now+Duration::minutes(minutes)));}
  Ok(at)
 }
 let result=Schedule{reminder_at:resolve(b.reminder_at,b.reminder_after_minutes,now)?,escalation_at:resolve(b.escalation_at,b.escalation_after_minutes,now)?,escalation_user_id:b.escalation_user_id};
 validate_schedule(&result,now)?;Ok(result)
}
pub fn validate_schedule(b:&Schedule,now:DateTime<Utc>)->Result<()> {
 if b.escalation_at.is_some()!=b.escalation_user_id.is_some(){return Err(ApiError::bad("Choose both an escalation person and time"));}
 for at in [b.reminder_at,b.escalation_at].into_iter().flatten(){if at<now+Duration::minutes(15)||at>now+Duration::days(30){return Err(ApiError::bad("Schedule follow-ups between 15 minutes and 30 days from now"));}}
 if let (Some(reminder),Some(escalation))=(b.reminder_at,b.escalation_at){if escalation<=reminder{return Err(ApiError::bad("Escalation must be after the reminder"));}}
 Ok(())
}
pub async fn schedule(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(input):Json<ScheduleInput>)->Result<Json<Value>> {
 let b=resolve_schedule(input,Utc::now())?;
 let mut tx=s.db.begin().await?;
 let row=crate::repositories::request_follow_up::schedule_select_service_requests(&tx, (id).to_owned()).await?;
 if row.try_get::<Uuid>("", "sender_user_id")?!=p.id{return Err(ApiError::forbidden());}
 if !["open","accepted"].contains(&row.try_get::<String>("", "status")?.as_str()){return Err(ApiError::conflict("Closed requests cannot have follow-ups"));}
 if let Some(user)=b.escalation_user_id {
  if user==p.id{return Err(ApiError::bad("Choose another person for escalation"));}
  if row.try_get::<String>("", "status")?!="open"{return Err(ApiError::conflict("This request has already been accepted; escalation is no longer needed"));}
  p.person(&s,user,"requests.send").await?;
  let active:bool=crate::repositories::request_follow_up::schedule_select_users(&tx, (user).to_owned()).await?;
  if !active{return Err(ApiError::bad("Choose an active escalation person"));}
 }
 crate::repositories::request_follow_up::schedule_insert_request_follow_ups(&tx, (id).to_owned(), (b.reminder_at).to_owned(), (b.escalation_at).to_owned(), (b.escalation_user_id).to_owned()).await?;
 let details=json!({"reminder_at":b.reminder_at,"escalation_at":b.escalation_at,"escalation_user_id":b.escalation_user_id});
 activity(&mut tx,id,Some(p.id),"follow_up_changed",details.clone()).await?;
 super::audit(&mut tx,p.id,"request.follow_up_changed","service_request",id,Value::Null,details).await?;
 tx.commit().await?;Ok(Json(json!({"ok":true})))
}
async fn remind(tx:&mut DatabaseTransaction,push:bool,id:Uuid,status:&str,assigned:Option<Uuid>,urgency:&str)->Result<u64> {
 let targets:Vec<Uuid>=crate::repositories::request_follow_up::remind_select_request_recipients(&*tx, (id).to_owned(), (status).to_owned(), (assigned).to_owned()).await?;
 if targets.is_empty(){return Ok(0);}
 for user in &targets{notify(tx,push,*user,id,"request_reminder",urgency).await?;}
 crate::repositories::request_follow_up::remind_update_service_requests(&*tx, (id).to_owned()).await?;
 Ok(targets.len() as u64)
}
pub async fn remind_now(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>> {
 let mut tx=s.db.begin().await?;
 let row=crate::repositories::request_follow_up::remind_now_select_service_requests(&tx, (id).to_owned()).await?;
 if row.try_get::<Uuid>("", "sender_user_id")?!=p.id{return Err(ApiError::forbidden());}
 let status:String=row.try_get("", "status")?;if !["open","accepted"].contains(&status.as_str()){return Err(ApiError::conflict("This request is already closed"));}
 if !row.try_get::<bool>("", "ready")?{return Err(ApiError::conflict("Allow at least 15 minutes after sending or the last reminder"));}
 let count=remind(&mut tx,s.push.is_some(),id,&status,row.try_get("", "assigned_to_user_id")?,&row.try_get::<String>("", "urgency")?).await?;
 if count==0{return Err(ApiError::conflict("There are no active recipients available to remind. You can cancel this request or arrange another recipient."));}
 activity(&mut tx,id,Some(p.id),"reminded",json!({"recipient_count":count})).await?;
 super::audit(&mut tx,p.id,"request.reminded","service_request",id,Value::Null,json!({"recipient_count":count})).await?;
 tx.commit().await?;Ok(Json(json!({"ok":true,"recipient_count":count})))
}
pub async fn run(s:AppState){loop{if let Err(error)=process_due(&s).await{tracing::error!(status=?error.0,"request follow-up cycle failed");}tokio::time::sleep(std::time::Duration::from_secs(30)).await;}}
// Public to allow deterministic database tests without spawning a loop or transport.
pub async fn process_due(s:&AppState)->Result<()> {
 for _ in 0..50 {
  let mut tx=s.db.begin().await?;
  // Always lock the request before its schedule, matching interactive operations.
  let row=crate::repositories::request_follow_up::process_due_select_service_requests(&tx).await?;
  let Some(row)=row else{break;};let id:Uuid=row.try_get("", "id")?;let sender:Uuid=row.try_get("", "sender_user_id")?;let status:String=row.try_get("", "status")?;let urgency:String=row.try_get("", "urgency")?;let now=Utc::now();
  if row.try_get::<Option<DateTime<Utc>>>("", "reminder_at")?.is_some_and(|t|t<=now)&&row.try_get::<Option<DateTime<Utc>>>("", "reminder_processed_at")?.is_none(){
   if ["open","accepted"].contains(&status.as_str())&&row.try_get::<Option<DateTime<Utc>>>("", "last_reminded_at")?.is_none_or(|t|t<=now-Duration::minutes(15)) {
    let count=remind(&mut tx,s.push.is_some(),id,&status,row.try_get("", "assigned_to_user_id")?,&urgency).await?;
    if count>0{activity(&mut tx,id,None,"reminded",json!({"recipient_count":count,"scheduled":true})).await?;}
    else{activity(&mut tx,id,None,"reminder_skipped",json!({"reason":"no_active_recipients"})).await?;}
   }else{activity(&mut tx,id,None,"reminder_skipped",json!({"reason":"closed_or_recent_reminder"})).await?;}
   crate::repositories::request_follow_up::process_due_update_request_follow_ups(&tx, (id).to_owned()).await?;
  }
  if row.try_get::<Option<DateTime<Utc>>>("", "escalation_at")?.is_some_and(|t|t<=now)&&row.try_get::<Option<DateTime<Utc>>>("", "escalation_processed_at")?.is_none(){
   let user:Uuid=row.try_get("", "escalation_user_id")?;
   // Re-check the sender's current scope and both account statuses at execution.
   let eligible:bool=crate::repositories::request_follow_up::process_due_select_users(&tx, (user).to_owned(), (sender).to_owned()).await?;
   if status=="open"&&eligible {
    crate::repositories::request_follow_up::process_due_insert_request_recipients(&tx, (id).to_owned(), (user).to_owned()).await?;
    notify(&mut tx,s.push.is_some(),user,id,"request_escalated",&urgency).await?;
    notify(&mut tx,s.push.is_some(),sender,id,"request_update",&urgency).await?;
    activity(&mut tx,id,None,"escalated",json!({"user_id":user})).await?;
    super::audit(&mut tx,sender,"request.escalated","service_request",id,Value::Null,json!({"user_id":user,"scheduled":true})).await?;
   }else{activity(&mut tx,id,None,"escalation_skipped",json!({"reason":if status!="open"{"already_accepted_or_closed"}else{"recipient_unavailable_or_scope_changed"}})).await?;}
   crate::repositories::request_follow_up::process_due_update_request_follow_ups_2(&tx, (id).to_owned()).await?;
  }
  tx.commit().await?;
 }Ok(())
}
