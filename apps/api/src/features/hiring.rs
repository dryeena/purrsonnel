use axum::{extract::{State,Path,Query},Json};
use chrono::{DateTime,Utc};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::{audit,Page};
pub async fn positions(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{
 let rows:Vec<Value>=crate::repositories::hiring::positions_select_applications(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Position {pub title:String,pub department_id:Uuid,pub position_level_id:Uuid,pub is_leadership:bool,pub description_markdown:String,pub hiring_owner_user_id:Uuid,pub status:String,pub questions:Option<Value>,pub form_template_version_id:Option<Uuid>}
pub async fn create_position(State(s):State<AppState>,p:Principal,Json(b):Json<Position>)->Result<Json<Value>>{
 p.require(&s,"positions.manage",Some(b.department_id)).await?;validation::text(&b.title,160)?;validation::text(&b.description_markdown,20000)?;if !["draft","open"].contains(&b.status.as_str()){return Err(ApiError::bad("Start as a draft or open posting"));}
 let owner_ok=Principal{id:b.hiring_owner_user_id}.can(&s,"interviews.finalize",Some(b.department_id)).await?;if !owner_ok{return Err(ApiError::bad("Hiring owner needs explicit finalization permission for this department"));}
 let mut tx=s.db.begin().await?;
 let mut questions:Vec<Value>=crate::repositories::hiring::create_position_select_organization_default_questions(&tx).await?;
 if let Some(version)=b.form_template_version_id{let schema:Value=crate::repositories::hiring::create_position_select_form_template_versions(&tx, (version).to_owned(), (b.department_id).to_owned()).await?;questions=schema.as_array().ok_or_else(||ApiError::bad("Invalid template version"))?.clone();}
 if let Some(custom)=b.questions{questions.extend(custom.as_array().ok_or_else(||ApiError::bad("Questions must be an array"))?.clone());}let schema=json!(questions);validation::validate_schema(&schema)?;
 let template:Uuid=crate::repositories::hiring::create_position_insert_form_templates(&tx, (&b.title).to_owned(), (b.department_id).to_owned(), (p.id).to_owned()).await?;
 let version:Uuid=crate::repositories::hiring::create_position_insert_form_template_versions(&tx, (template).to_owned(), (schema).to_owned()).await?;
 let id:Uuid=crate::repositories::hiring::create_position_insert_position_postings(&tx, (&b.title).to_owned(), (b.department_id).to_owned(), (b.position_level_id).to_owned(), (b.is_leadership).to_owned(), (b.description_markdown).to_owned(), (b.hiring_owner_user_id).to_owned(), (b.status).to_owned(), (p.id).to_owned(), (version).to_owned()).await?;audit(&mut tx,p.id,"position.created","position",id,Value::Null,json!({"title":b.title})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct PositionPatch {pub status:String}
pub async fn update_position(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<PositionPatch>)->Result<Json<Value>>{let dept:Uuid=crate::repositories::hiring::update_position_select_position_postings(&s.db, (id).to_owned()).await?;p.require(&s,"positions.manage",Some(dept)).await?;if !["open","closed","archived","draft"].contains(&b.status.as_str()){return Err(ApiError::bad("Invalid posting status"));}let mut tx=s.db.begin().await?;crate::repositories::hiring::update_position_update_position_postings(&tx, (id).to_owned(), (&b.status).to_owned()).await?;audit(&mut tx,p.id,"position.status_changed","position",id,Value::Null,json!({"status":b.status})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
pub async fn questions(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{p.require(&s,"positions.manage",None).await?;Ok(Json(json!(crate::repositories::hiring::questions_select_organization_default_questions::<_,Value>(&s.db).await?)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Question{pub key:String,pub label:String,pub r#type:String,pub required:bool,pub sensitive:bool}
pub async fn create_question(State(s):State<AppState>,p:Principal,Json(b):Json<Question>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;validation::validate_schema(&json!([{"key":b.key,"label":b.label,"type":b.r#type}]))?;let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::hiring::create_question_insert_organization_default_questions(&tx, (&b.key).to_owned(), (b.label).to_owned(), (b.r#type).to_owned(), (b.required).to_owned(), (b.sensitive).to_owned()).await?;audit(&mut tx,p.id,"default_question.created","question",id,Value::Null,json!({"key":b.key})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
pub async fn applications(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{
 // Answers are returned by the single-record endpoint, where sensitive fields can be redacted.
 let rows:Vec<Value>=crate::repositories::hiring::applications_select_applications(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Apply {pub position_posting_id:Uuid,pub answers_json:Value}
pub async fn apply(State(s):State<AppState>,p:Principal,Json(b):Json<Apply>)->Result<Json<Value>>{
 let mut tx=s.db.begin().await?;let dept:Uuid=crate::repositories::hiring::apply_select_position_postings(&tx, (b.position_posting_id).to_owned()).await?;p.require(&s,"applications.submit",Some(dept)).await?;
 if !b.answers_json.is_object()||b.answers_json.to_string().len()>100000{return Err(ApiError::bad("Invalid draft answers"));}
 let id:Uuid=crate::repositories::hiring::apply_insert_applications(&tx, (b.position_posting_id).to_owned(), (p.id).to_owned(), (b.answers_json).to_owned()).await?;audit(&mut tx,p.id,"application.drafted","application",id,Value::Null,json!({"position_id":b.position_posting_id})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
pub async fn application(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{
 let row=crate::repositories::hiring::application_select_applications(&s.db, (id).to_owned()).await?;
 let own=row.try_get::<Uuid>("", "applicant_user_id")?==p.id;if !own{p.require(&s,"applications.review",Some(row.try_get("", "department_id")?)).await?;}
 let mut data:Value=row.try_get("", "data")?;if !own&&!p.can(&s,"people.read_sensitive",Some(row.try_get("", "department_id")?)).await?{let schema:Value=row.try_get("", "schema_json")?;for f in schema.as_array().into_iter().flatten(){if f["sensitive"]==true{if let Some(obj)=data["answers_json"].as_object_mut(){obj.remove(f["key"].as_str().unwrap_or(""));}}}}
 Ok(Json(data))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Answers{pub answers_json:Value}
pub async fn edit_application(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Answers>)->Result<Json<Value>>{if !b.answers_json.is_object()||b.answers_json.to_string().len()>100000{return Err(ApiError::bad("Invalid draft answers"));}let n=crate::repositories::hiring::edit_application_update_applications(&s.db, (id).to_owned(), (p.id).to_owned(), (b.answers_json).to_owned()).await?.rows_affected();if n==0{return Err(ApiError::conflict("Only your own draft can be edited"));}Ok(Json(json!({"ok":true})))}
pub async fn submit_application(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let mut tx=s.db.begin().await?;let row=crate::repositories::hiring::submit_application_select_applications(&tx, (id).to_owned()).await?;if row.try_get::<Uuid>("", "applicant_user_id")?!=p.id{return Err(ApiError::forbidden());}p.require(&s,"applications.submit",Some(row.try_get("", "department_id")?)).await?;
 if row.try_get::<String>("", "status")?!="draft"||row.try_get::<String>("", "posting_status")?!="open"||row.try_get::<Option<DateTime<Utc>>>("", "closes_at")?.is_some_and(|d|d<Utc::now()){return Err(ApiError::conflict("Application must be a draft for an open posting"));}
 validation::validate_answers(&row.try_get("", "schema_json")?,&row.try_get("", "answers_json")?)?;
 crate::repositories::hiring::submit_application_update_applications(&tx, (id).to_owned(), (p.id).to_owned()).await?;audit(&mut tx,p.id,"application.submitted","application",id,Value::Null,json!({"status":"submitted"})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
pub async fn interviews(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{
 let rows:Vec<Value>=crate::repositories::hiring::interviews_select_interview_participants(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Schedule{pub application_id:Uuid,pub scheduled_start_at:DateTime<Utc>,pub scheduled_end_at:DateTime<Utc>,pub timezone:String,pub location_label:String,pub location_url:Option<String>,pub panel:Vec<Uuid>}
pub async fn schedule(State(s):State<AppState>,p:Principal,Json(b):Json<Schedule>)->Result<Json<Value>>{
 let dept:Uuid=crate::repositories::hiring::schedule_select_applications(&s.db, (b.application_id).to_owned()).await?;p.require(&s,"interviews.manage",Some(dept)).await?;
 validation::text(&b.location_label,200)?;if b.scheduled_end_at<=b.scheduled_start_at||(b.scheduled_end_at-b.scheduled_start_at).num_hours()>8{return Err(ApiError::bad("Invalid interview duration"));}let tz:bool=crate::repositories::hiring::schedule_select_pg_timezone_names(&s.db, (&b.timezone).to_owned()).await?;if !tz{return Err(ApiError::bad("Use an IANA timezone"));}if b.location_url.as_ref().is_some_and(|u|!u.starts_with("https://")||u.len()>2000){return Err(ApiError::bad("Meeting URLs must use HTTPS"));}if b.panel.is_empty()||b.panel.len()>12{return Err(ApiError::bad("Choose 1–12 interviewers"));}
 for id in &b.panel{if !(Principal{id:*id}).can(&s,"interviews.decide",Some(dept)).await?{return Err(ApiError::bad("Each panel member needs interviews.decide in this department"));}}
 let mut tx=s.db.begin().await?;
 // Revalidate state under lock: finalization and scheduling cannot race.
 let state:String=crate::repositories::hiring::schedule_select_applications_2(&tx, (b.application_id).to_owned()).await?;if !["submitted","screening","interviewing"].contains(&state.as_str()){return Err(ApiError::conflict("Application is no longer available to schedule"));}
 let id:Uuid=crate::repositories::hiring::schedule_insert_interviews(&tx, (b.application_id).to_owned(), (b.scheduled_start_at).to_owned(), (b.scheduled_end_at).to_owned(), (b.timezone).to_owned(), (b.location_label).to_owned(), (b.location_url).to_owned(), (p.id).to_owned()).await?;
 for member in b.panel {crate::repositories::hiring::schedule_insert_interview_participants(&tx, (id).to_owned(), (member).to_owned(), (p.id).to_owned()).await?;}
 crate::repositories::hiring::schedule_update_applications(&tx, (b.application_id).to_owned()).await?;audit(&mut tx,p.id,"interview.scheduled","interview",id,Value::Null,json!({"application_id":b.application_id})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Recommendation {pub decision:String,pub note:String}
pub async fn recommend(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Recommendation>)->Result<Json<Value>>{
 validation::decision(&b.decision,&b.note)?;let dept:Uuid=crate::repositories::hiring::recommend_select_interviews(&s.db, (id).to_owned()).await?;p.require(&s,"interviews.decide",Some(dept)).await?;
 let mut tx=s.db.begin().await?;let n=crate::repositories::hiring::recommend_update_interview_participants(&tx, (id).to_owned(), (p.id).to_owned(), (&b.decision).to_owned(), (b.note).to_owned()).await?.rows_affected();if n==0{return Err(ApiError::forbidden());}audit(&mut tx,p.id,"interview.recommended","interview",id,Value::Null,json!({"decision":b.decision})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
pub async fn notes(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{
 let rows:Vec<Value>=crate::repositories::hiring::notes_select_interview_notes(&s.db, (id).to_owned(), (p.id).to_owned()).await?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Note{pub body:String,pub visibility:String}
pub async fn add_note(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Note>)->Result<Json<Value>>{
 validation::text(&b.body,10000)?;if !["author_only","interview_panel","hr"].contains(&b.visibility.as_str()){return Err(ApiError::bad("Invalid note visibility"));}
 let permitted:bool=crate::repositories::hiring::add_note_select_interviews(&s.db, (id).to_owned(), (p.id).to_owned()).await?;if !permitted{return Err(ApiError::forbidden());}
 let mut tx=s.db.begin().await?;let note:Uuid=crate::repositories::hiring::add_note_insert_interview_notes(&tx, (id).to_owned(), (p.id).to_owned(), (b.body).to_owned(), (&b.visibility).to_owned()).await?;audit(&mut tx,p.id,"interview.note_added","interview",id,Value::Null,json!({"note_id":note,"visibility":b.visibility})).await?;tx.commit().await?;Ok(Json(json!({"id":note})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Finalize {pub decision:String,pub reason:String,pub idempotency_key:Uuid}
pub async fn finalize(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Finalize>)->Result<Json<Value>>{
 validation::decision(&b.decision,&b.reason)?;validation::text(&b.reason,2000)?;
 let mut tx=s.db.begin().await?;let row=crate::repositories::hiring::finalize_select_applications(&tx, (id).to_owned()).await?;
 let dept:Uuid=row.try_get("", "department_id")?;p.require(&s,"interviews.finalize",Some(dept)).await?;
 if row.try_get::<Uuid>("", "hiring_owner_user_id")?!=p.id{p.require(&s,"interviews.finalize",None).await?;}
 if let Some(prev)=crate::repositories::hiring::finalize_select_application_final_decisions(&tx, (b.idempotency_key).to_owned()).await?{if prev.try_get::<Uuid>("", "application_id")?==id&&prev.try_get::<Uuid>("", "decided_by_user_id")?==p.id&&prev.try_get::<String>("", "decision")?==b.decision&&prev.try_get::<String>("", "reason")?==b.reason{return Ok(Json(json!({"ok":true,"replayed":true})));}return Err(ApiError::conflict("Idempotency key was already used"));}
 if !["submitted","screening","interviewing","offered"].contains(&row.try_get::<String>("", "status")?.as_str()){return Err(ApiError::conflict("This application cannot be finalized"));}
 let user:Uuid=row.try_get("", "applicant_user_id")?;
 if b.decision=="approve"{
  let level:Uuid=row.try_get::<Option<Uuid>>("", "personnel_level_id")?.ok_or_else(||ApiError::bad("Other requires an explicit personnel level mapping before hiring"))?;
  crate::repositories::hiring::finalize_select_users(&tx, (user).to_owned()).await?;
  // Preserve higher existing rank and identity state; never grant permissions or a login.
  crate::repositories::hiring::finalize_update_users(&tx, (user).to_owned(), (level).to_owned()).await?;
  crate::repositories::hiring::finalize_insert_user_departments(&tx, (user).to_owned(), (dept).to_owned()).await?;
 }
 crate::repositories::hiring::finalize_insert_application_final_decisions(&tx, (id).to_owned(), (&b.decision).to_owned(), (p.id).to_owned(), (b.reason).to_owned(), (b.idempotency_key).to_owned()).await?;
 let status=if b.decision=="approve"{"approved"}else{"rejected"};crate::repositories::hiring::finalize_update_applications(&tx, (id).to_owned(), (status).to_owned(), (p.id).to_owned()).await?;audit(&mut tx,p.id,"application.finalized","application",id,Value::Null,json!({"decision":b.decision,"applicant_user_id":user,"department_id":dept})).await?;
 crate::repositories::hiring::finalize_insert_notifications(&tx, (user).to_owned(), (json!({"application_id":id,"status":status})).to_owned()).await?;tx.commit().await?;Ok(Json(json!({"ok":true,"status":status,"user_id":user})))
}
