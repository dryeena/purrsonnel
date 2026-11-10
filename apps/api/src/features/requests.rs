//! Requests are delivered to a fixed recipient snapshot. A team member claims a
//! request before completion; concurrent claimers cannot both take ownership.
use axum::{extract::{State,Path,Query},Json};
use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::{Principal,hash},error::{ApiError,Result},validation};
use super::{audit,Page};

pub async fn recipients(State(s):State<AppState>,p:Principal)->Result<Json<Value>> {
 let departments:Vec<Value>=crate::repositories::requests::recipients_select_user_departments(&s.db, (p.id).to_owned()).await?;
 let people:Vec<Value>=crate::repositories::requests::recipients_select_users(&s.db, (p.id).to_owned()).await?;
 Ok(Json(json!({"departments":departments,"people":people})))
}

#[derive(Deserialize,Serialize)]#[serde(deny_unknown_fields)]
pub struct Create {
 pub recipient_user_id:Option<Uuid>, pub recipient_department_id:Option<Uuid>,
 pub title:String,pub body:String,pub urgency:String,pub due_at:Option<DateTime<Utc>>,pub idempotency_key:Uuid,
}
pub fn validate_create(b:&Create)->Result<()> {
 validation::text(&b.title,160)?;validation::text(&b.body,4000)?;
 if b.recipient_user_id.is_some()==b.recipient_department_id.is_some(){return Err(ApiError::bad("Choose exactly one team or person"));}
 if !["normal","urgent"].contains(&b.urgency.as_str()){return Err(ApiError::bad("Choose normal or urgent priority"));}
 Ok(())
}
#[derive(Deserialize)]pub struct RequestPage{#[serde(flatten)]pub page:Page,pub view:Option<String>,pub status:Option<String>,pub q:Option<String>}
pub async fn create(State(s):State<AppState>,p:Principal,Json(b):Json<Create>)->Result<Json<Value>> {
 validate_create(&b)?;
 if let Some(dept)=b.recipient_department_id{p.require(&s,"requests.send",Some(dept)).await?;}
 if let Some(person)=b.recipient_user_id{if person==p.id{return Err(ApiError::bad("Choose another person"));}p.person(&s,person,"requests.send").await?;}
 let digest=hash(&serde_json::to_string(&b).map_err(|_|ApiError::bad("Invalid request"))?);
 let mut tx=s.db.begin().await?;
 // Lock the audience against membership/person changes until the snapshot commits.
 let recipients:Vec<Uuid>=if let Some(dept)=b.recipient_department_id {
  crate::repositories::requests::create_select_departments(&tx, (dept).to_owned()).await?;
  crate::repositories::requests::create_select_users(&tx, (dept).to_owned(), (p.id).to_owned()).await?
 } else {
  vec![crate::repositories::requests::create_select_users_2(&tx, (b.recipient_user_id).to_owned()).await?]
 };
 if recipients.is_empty()||recipients.len()>250{return Err(ApiError::bad("Choose a recipient or a team with 1–250 other active members"));}
 let new_id:Option<Uuid>=crate::repositories::requests::create_insert_service_requests(&tx, (p.id).to_owned(), (b.recipient_user_id).to_owned(), (b.recipient_department_id).to_owned(), (&b.title).to_owned(), (&b.body).to_owned(), (&b.urgency).to_owned(), (b.due_at).to_owned(), (b.idempotency_key).to_owned(), (&digest).to_owned()).await?;
 let Some(id)=new_id else {
  let prior=crate::repositories::requests::create_select_service_requests(&tx, (p.id).to_owned(), (b.idempotency_key).to_owned()).await?;
  if prior.try_get::<String>("", "payload_sha256")?!=digest{return Err(ApiError::conflict("This submission key belongs to a different request"));}
  return Ok(Json(json!({"id":prior.try_get::<Uuid>("", "id")?,"replayed":true})));
 };
 let mut queued=0u64;let mut suppressed=0u64;
 for user in &recipients {
  crate::repositories::requests::create_insert_request_recipients(&tx, (id).to_owned(), (user).to_owned()).await?;
  let (pending,skipped)=super::request_follow_up::notify(&mut tx,s.push.is_some(),*user,id,"service_request",&b.urgency).await?;
  queued+=pending;suppressed+=skipped;
 }
 audit(&mut tx,p.id,"request.sent","service_request",id,Value::Null,json!({"recipient_count":recipients.len(),"recipient_department_id":b.recipient_department_id,"recipient_user_id":b.recipient_user_id,"push_queued":queued})).await?;
 super::request_follow_up::activity(&mut tx,id,Some(p.id),"sent",json!({"recipient_count":recipients.len()})).await?;
 tx.commit().await?;
 Ok(Json(json!({"id":id,"recipient_count":recipients.len(),"push_queued":queued,"push_suppressed":suppressed,"push_enabled":s.push.is_some()})))
}

pub async fn list(State(s):State<AppState>,p:Principal,Query(query):Query<RequestPage>)->Result<Json<Value>> {
 let view=query.view.as_deref().unwrap_or("all");if !["all","inbox","sent"].contains(&view){return Err(ApiError::bad("Unknown inbox view"));}let page=query.page;
 let status=query.status.as_deref().unwrap_or("all");if !["all","active","open","accepted","completed","cancelled"].contains(&status){return Err(ApiError::bad("Unknown request status"));}
 let search=query.q.as_deref().unwrap_or("").trim();if search.chars().count()>160{return Err(ApiError::bad("Search must be 160 characters or fewer"));}
 let search_pattern=format!("%{}%",search.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_"));
 let rows:Vec<Value>=crate::repositories::requests::list_select_request_recipients(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned(), (view).to_owned(), (status).to_owned(), (search_pattern).to_owned()).await?;
 Ok(Json(json!(rows)))
}
pub async fn get(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>> {
 let data:Value=crate::repositories::requests::get_select_request_recipients(&s.db, (id).to_owned(), (p.id).to_owned()).await?;
 Ok(Json(data))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Respond {pub action:String}
pub async fn respond(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Respond>)->Result<Json<Value>> {
 if !["accept","complete","cancel"].contains(&b.action.as_str()){return Err(ApiError::bad("Choose accept, complete, or cancel"));}
 let mut tx=s.db.begin().await?;
 let row=crate::repositories::requests::respond_select_service_requests(&tx, (id).to_owned()).await?;
 let recipient:bool=crate::repositories::requests::respond_select_request_recipients(&tx, (id).to_owned(), (p.id).to_owned()).await?;
 let sender:Uuid=row.try_get("", "sender_user_id")?;let status:String=row.try_get("", "status")?;let assigned:Option<Uuid>=row.try_get("", "assigned_to_user_id")?;
 let next=match b.action.as_str(){
  "accept" if recipient&&status=="open"=>"accepted",
  "complete" if (assigned==Some(p.id)||sender==p.id)&&status=="accepted"=>"completed",
  "cancel" if sender==p.id&&["open","accepted"].contains(&status.as_str())=>"cancelled",
  _=>return Err(if !recipient&&sender!=p.id{ApiError::forbidden()}else{ApiError::conflict("This request has changed, or another teammate has accepted it. Refresh to see its current status.")})
 };
 crate::repositories::requests::respond_update_service_requests(&tx, (id).to_owned(), (next).to_owned(), (p.id).to_owned()).await?;
 crate::repositories::requests::respond_update_request_recipients(&tx, (id).to_owned(), (p.id).to_owned()).await?;
 let observers:Vec<Uuid>=crate::repositories::requests::respond_select_request_recipients_2(&tx, (id).to_owned(), (p.id).to_owned(), (sender).to_owned()).await?;
 for user in observers{super::request_follow_up::notify(&mut tx,s.push.is_some(),user,id,"request_update",&row.try_get::<String>("", "urgency")?).await?;}
 super::request_follow_up::activity(&mut tx,id,Some(p.id),next,json!({})).await?;
 audit(&mut tx,p.id,"request.responded","service_request",id,json!({"status":status}),json!({"status":next})).await?;tx.commit().await?;
 Ok(Json(json!({"ok":true,"status":next})))
}
