use axum::{extract::{State,Path,Query},Json};
use chrono::{DateTime,Utc,NaiveDate};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::{Principal,hash,token},error::{ApiError,Result},validation};
use super::{audit,Page};
pub async fn list(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{let rows:Vec<Value>=crate::repositories::volunteers::list_select_volunteer_entries(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Entry {pub department_id:Uuid,pub event_edition_id:Option<Uuid>,pub work_date:NaiveDate,pub minutes_worked:Option<i32>,pub started_at:Option<DateTime<Utc>>,pub ended_at:Option<DateTime<Utc>>,pub work_description:String}
pub fn duration(b:&Entry)->Result<i32>{
 let minutes=match (b.started_at,b.ended_at,b.minutes_worked){(Some(start),Some(end),None)=>(end-start).num_minutes(),(None,None,Some(n))=>n as i64,_=>return Err(ApiError::bad("Supply a duration OR timezone-aware start and end times"))};
 if !(1..=960).contains(&minutes){return Err(ApiError::bad("Duration must be between 1 minute and 16 hours"));}Ok(minutes as i32)
}
pub async fn create(State(s):State<AppState>,p:Principal,Json(b):Json<Entry>)->Result<Json<Value>>{
 p.require(&s,"volunteer.time.submit",Some(b.department_id)).await?;validation::text(&b.work_description,2000)?;let minutes=duration(&b)?;
 if minutes>s.config.max_volunteer_minutes{return Err(ApiError::bad(format!("Maximum duration is {} minutes",s.config.max_volunteer_minutes)));}
 if b.work_date>Utc::now().date_naive(){return Err(ApiError::bad("Hours cannot be recorded in the future"));}
 if b.ended_at.is_some_and(|end|end>Utc::now()){return Err(ApiError::bad("End time cannot be in the future"));}
 let eligible:bool=crate::repositories::volunteers::create_select_users(&s.db, (p.id).to_owned()).await?;if !eligible{return Err(ApiError::bad("A linked, volunteer-eligible identity is required"));}
 let mut tx=s.db.begin().await?;crate::repositories::volunteers::create_select_users_2(&tx, (p.id).to_owned()).await?;
 let overlap:bool=crate::repositories::volunteers::create_select_volunteer_entries(&tx, (p.id).to_owned(), (b.started_at).to_owned(), (b.ended_at).to_owned()).await?;if overlap{return Err(ApiError::conflict("These times overlap an existing entry"));}
 let id:Uuid=crate::repositories::volunteers::create_insert_volunteer_entries(&tx, (p.id).to_owned(), (b.event_edition_id).to_owned(), (b.department_id).to_owned(), (b.work_date).to_owned(), (minutes).to_owned(), (b.started_at).to_owned(), (b.ended_at).to_owned(), (b.work_description).to_owned()).await?;audit(&mut tx,p.id,"volunteer.drafted","volunteer_entry",id,Value::Null,json!({"minutes":minutes})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
pub async fn submit(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{transition(s,p,id,"submitted").await}
pub async fn withdraw(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{transition(s,p,id,"withdrawn").await}
async fn transition(s:AppState,p:Principal,id:Uuid,status:&str)->Result<Json<Value>>{
 let mut tx=s.db.begin().await?;
 let eligible:bool=crate::repositories::volunteers::transition_select_users(&tx, (p.id).to_owned()).await?;
 if status=="submitted"&&!eligible{return Err(ApiError::bad("A linked, eligible volunteer identity is required before submission"));}
 let row=crate::repositories::volunteers::transition_select_volunteer_entries(&tx, (id).to_owned()).await?;
 if row.try_get::<Uuid>("", "volunteer_user_id")?!=p.id{return Err(ApiError::forbidden());}p.require(&s,"volunteer.time.submit",Some(row.try_get("", "department_id")?)).await?;
 let current:String=row.try_get("", "status")?;if (status=="submitted"&&current!="draft")||(status=="withdrawn"&&! ["draft","submitted"].contains(&current.as_str())){return Err(ApiError::conflict("This entry cannot make that transition"));}
 crate::repositories::volunteers::transition_update_volunteer_entries(&tx, (id).to_owned(), (status).to_owned()).await?;
 crate::repositories::volunteers::transition_update_volunteer_approval_tokens(&tx, (id).to_owned()).await?;audit(&mut tx,p.id,&format!("volunteer.{status}"),"volunteer_entry",id,json!({"status":current}),json!({"status":status})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
pub async fn qr(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let mut tx=s.db.begin().await?;let row=crate::repositories::volunteers::qr_select_volunteer_entries(&tx, (id).to_owned()).await?;if row.try_get::<Uuid>("", "volunteer_user_id")?!=p.id{return Err(ApiError::forbidden());}if row.try_get::<String>("", "status")?!="submitted"{return Err(ApiError::conflict("Only submitted entries can have an approval code"));}crate::repositories::volunteers::qr_update_volunteer_approval_tokens(&tx, (id).to_owned()).await?;let raw=token();let expiry:DateTime<Utc>=crate::repositories::volunteers::qr_insert_volunteer_approval_tokens(&tx, (id).to_owned(), (hash(&raw)).to_owned()).await?;tx.commit().await?;Ok(Json(json!({"url":format!("{}/volunteer/approve?t={raw}",s.config.origin),"expires_at":expiry})))}
#[derive(Deserialize)]pub struct TokenQuery {pub token:String}
async fn require_staff_approver(s:&AppState,id:Uuid)->Result<()>{if crate::repositories::volunteers::approver_select_users::<_,bool>(&s.db,id).await?{Ok(())}else{Err(ApiError::forbidden())}}
pub async fn preview(State(s):State<AppState>,p:Principal,Query(q):Query<TokenQuery>)->Result<Json<Value>>{let row=crate::repositories::volunteers::preview_select_volunteer_approval_tokens(&s.db, (hash(&q.token)).to_owned()).await?.ok_or_else(||ApiError::conflict("Code expired, used, or revoked. Ask for a new code."))?;require_staff_approver(&s,p.id).await?;if row.try_get::<Uuid>("", "volunteer_user_id")?==p.id{return Err(ApiError::forbidden());}Ok(Json(row.try_get("", "data")?))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Decision {pub token:String,pub decision:String,pub note:String,pub idempotency_key:Uuid}
pub async fn decide(State(s):State<AppState>,p:Principal,Json(b):Json<Decision>)->Result<Json<Value>>{
 validation::decision(&b.decision,&b.note)?;
 let mut tx=s.db.begin().await?;
 // Always lock the entry before its token: same order as regeneration and withdrawal.
 let id:Uuid=crate::repositories::volunteers::decide_select_volunteer_approval_tokens(&tx, (hash(&b.token)).to_owned()).await?;
 let entry=crate::repositories::volunteers::decide_select_volunteer_entries(&tx, (id).to_owned()).await?;
 require_staff_approver(&s,p.id).await?;if entry.try_get::<Uuid>("", "volunteer_user_id")?==p.id{return Err(ApiError::forbidden());}
 if let Some(previous)=crate::repositories::volunteers::decide_select_volunteer_decision_events(&tx, (b.idempotency_key).to_owned()).await?{let expected=if b.decision=="approve"{"approved"}else{"rejected"};if previous.try_get::<Uuid>("", "entry_id")?==id && previous.try_get::<Uuid>("", "actor_user_id")?==p.id && previous.try_get::<String>("", "decision")?==expected && previous.try_get::<String>("", "note")?==b.note && previous.try_get::<Option<String>>("", "token_hash")?==Some(hash(&b.token)){return Ok(Json(json!({"ok":true,"replayed":true})));}return Err(ApiError::conflict("Idempotency key was used for another operation"));}
 let valid:bool=crate::repositories::volunteers::decide_select_volunteer_approval_tokens_2(&tx, (hash(&b.token)).to_owned()).await?;
 if !valid||entry.try_get::<String>("", "status")?!="submitted"{return Err(ApiError::conflict("This approval code is no longer usable"));}
 let status=if b.decision=="approve"{"approved"}else{"rejected"};
 crate::repositories::volunteers::decide_update_volunteer_entries(&tx, (id).to_owned(), (status).to_owned(), (p.id).to_owned(), (&b.note).to_owned()).await?;
 crate::repositories::volunteers::decide_update_volunteer_approval_tokens(&tx, (hash(&b.token)).to_owned(), (p.id).to_owned()).await?;
 crate::repositories::volunteers::decide_update_volunteer_approval_tokens_2(&tx, (id).to_owned()).await?;
 crate::repositories::volunteers::decide_insert_volunteer_decision_events(&tx, (id).to_owned(), (p.id).to_owned(), (status).to_owned(), (&b.note).to_owned(), "qr", (b.idempotency_key).to_owned(), Some(hash(&b.token))).await?;
 audit(&mut tx,p.id,&format!("volunteer.{status}"),"volunteer_entry",id,json!({"status":"submitted"}),json!({"status":status})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct ManualDecision {pub entry_id:Uuid,pub decision:String,pub note:String,pub idempotency_key:Uuid}
pub async fn decide_manual(State(s):State<AppState>,p:Principal,Json(b):Json<ManualDecision>)->Result<Json<Value>>{
 validation::decision(&b.decision,&b.note)?;
 let mut tx=s.db.begin().await?;
 let entry=crate::repositories::volunteers::decide_select_volunteer_entries(&tx, (b.entry_id).to_owned()).await?;
 require_staff_approver(&s,p.id).await?;
 if entry.try_get::<Uuid>("", "volunteer_user_id")?==p.id{return Err(ApiError::forbidden());}
 let status=if b.decision=="approve"{"approved"}else{"rejected"};
 if let Some(previous)=crate::repositories::volunteers::decide_select_volunteer_decision_events(&tx, (b.idempotency_key).to_owned()).await?{
  if previous.try_get::<Uuid>("", "entry_id")?==b.entry_id&&previous.try_get::<Uuid>("", "actor_user_id")?==p.id&&previous.try_get::<String>("", "decision")?==status&&previous.try_get::<String>("", "note")?==b.note&&previous.try_get::<Option<String>>("", "token_hash")?.is_none(){return Ok(Json(json!({"ok":true,"replayed":true})));}
  return Err(ApiError::conflict("Idempotency key was used for another operation"));
 }
 if entry.try_get::<String>("", "status")? != "submitted" {return Err(ApiError::conflict("Only submitted entries can be decided"));}
 crate::repositories::volunteers::decide_update_volunteer_entries(&tx, (b.entry_id).to_owned(), status, (p.id).to_owned(), (&b.note).to_owned()).await?;
 crate::repositories::volunteers::decide_update_volunteer_approval_tokens_2(&tx, (b.entry_id).to_owned()).await?;
 crate::repositories::volunteers::decide_insert_volunteer_decision_events(&tx, (b.entry_id).to_owned(), (p.id).to_owned(), status, (&b.note).to_owned(), "manual", (b.idempotency_key).to_owned(), Option::<String>::None).await?;
 audit(&mut tx,p.id,&format!("volunteer.{status}"),"volunteer_entry",b.entry_id,json!({"status":"submitted"}),json!({"status":status,"method":"manual"})).await?;
 tx.commit().await?;Ok(Json(json!({"ok":true})))
}
pub async fn report(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{let rows:Vec<Value>=crate::repositories::volunteers::report_select_volunteer_entries(&s.db, (p.id).to_owned()).await?;Ok(Json(json!(rows)))}
