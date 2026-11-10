use sea_orm::TransactionTrait;
use axum::{extract::State,Json};
use serde::Deserialize;
use serde_json::{json,Value};
use uuid::Uuid;
use chrono::{DateTime,Utc};
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::audit;
pub async fn tree(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{
 let departments:Vec<Value>=crate::repositories::organization::tree_select_user_departments(&s.db).await?;
 let divisions:Vec<Value>=crate::repositories::organization::tree_select_divisions(&s.db).await?;
 let levels:Vec<Value>=crate::repositories::organization::tree_select_personnel_levels(&s.db).await?;
 let position_levels:Vec<Value>=crate::repositories::organization::tree_select_position_levels(&s.db).await?;
 let events:Vec<Value>=crate::repositories::organization::tree_select_event_editions(&s.db).await?;
 // Counts are personnel information: remove them for identities without scoped read access.
 let mut departments=departments;for d in &mut departments {let dep=Uuid::parse_str(d["id"].as_str().unwrap_or("")).ok();if !p.can(&s,"people.read",dep).await?{d.as_object_mut().unwrap().remove("member_count");}}
 Ok(Json(json!({"departments":departments,"divisions":divisions,"levels":levels,"position_levels":position_levels,"events":events,"volunteer_max_minutes":s.config.max_volunteer_minutes})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Department {pub name:String,pub slug:String,pub division_id:Option<Uuid>,pub description:Option<String>,pub lead_user_id:Option<Uuid>}
pub async fn create_department(State(s):State<AppState>,p:Principal,Json(b):Json<Department>)->Result<Json<Value>>{p.require(&s,"organization.manage",None).await?;validation::text(&b.name,100)?;validation::text(&b.slug,100)?;let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::organization::create_department_insert_departments(&tx, (&b.name).to_owned(), (b.slug).to_owned(), (b.division_id).to_owned(), (b.description.unwrap_or_default()).to_owned(), (b.lead_user_id).to_owned()).await?;audit(&mut tx,p.id,"department.created","department",id,Value::Null,json!({"name":b.name})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
#[derive(Deserialize)]pub struct Division {pub name:String,pub description:Option<String>}
pub async fn create_division(State(s):State<AppState>,p:Principal,Json(b):Json<Division>)->Result<Json<Value>>{p.require(&s,"organization.manage",None).await?;validation::text(&b.name,100)?;let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::organization::create_division_insert_divisions(&tx, (&b.name).to_owned(), (b.description.unwrap_or_default()).to_owned()).await?;audit(&mut tx,p.id,"division.created","division",id,Value::Null,json!({"name":b.name})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
pub async fn events(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;let rows:Vec<Value>=crate::repositories::organization::events_select_event_editions(&s.db).await?;Ok(Json(json!(rows)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Event {pub name:String,pub year:i32,pub starts_at:Option<DateTime<Utc>>,pub ends_at:Option<DateTime<Utc>>,pub active:Option<bool>}
pub async fn create_event(State(s):State<AppState>,p:Principal,Json(b):Json<Event>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;validation::text(&b.name,120)?;if !(2000..=2100).contains(&b.year)||b.ends_at.zip(b.starts_at).is_some_and(|(end,start)|end<start){return Err(ApiError::bad("Provide a valid event year and date range"));}let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::organization::events_insert_event_editions(&tx,b.name,b.year,b.starts_at,b.ends_at,b.active.unwrap_or(true)).await?;audit(&mut tx,p.id,"event.created","event",id,Value::Null,json!({"year":b.year})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct EventUpdate {pub active:bool}
pub async fn update_event(State(s):State<AppState>,p:Principal,axum::extract::Path(id):axum::extract::Path<Uuid>,Json(b):Json<EventUpdate>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;let mut tx=s.db.begin().await?;if crate::repositories::organization::events_update_event_editions(&tx,id,b.active).await?.rows_affected()==0{return Err(ApiError::missing());}audit(&mut tx,p.id,"event.updated","event",id,Value::Null,json!({"active":b.active})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
pub async fn fields(State(s):State<AppState>,p:Principal)->Result<Json<Value>> {p.require(&s,"settings.manage",None).await?;Ok(Json(json!(crate::repositories::organization::fields_select_profile_field_definitions::<_,Value>(&s.db).await?)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Field {pub key:String,pub label:String,pub r#type:String,pub choices:Option<Value>,pub visibility:Option<String>,pub editable_by:Option<String>,pub max_length:Option<u32>}
pub async fn create_field(State(s):State<AppState>,p:Principal,Json(b):Json<Field>)->Result<Json<Value>>{
 p.require(&s,"settings.manage",None).await?;validation::text(&b.key,60)?;validation::text(&b.label,120)?;
 if !["text","number","date","boolean","single_select","multi_select","url"].contains(&b.r#type.as_str()){return Err(ApiError::bad("Unsupported profile field type"));}
 let choices=b.choices.unwrap_or(json!([]));validation::validate_schema(&json!([{"key":b.key,"label":b.label,"type":b.r#type,"options":choices}]))?;
 let visibility=b.visibility.unwrap_or_else(||"self".into());let editable=b.editable_by.unwrap_or_else(||"hr".into());
 if !["self","personnel","sensitive"].contains(&visibility.as_str())||!["self","hr"].contains(&editable.as_str())||b.max_length.is_some_and(|m|m==0||m>10000){return Err(ApiError::bad("Invalid profile field policy"));}
 let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::organization::create_field_insert_profile_field_definitions(&tx, (&b.key).to_owned(), (b.label).to_owned(), (b.r#type).to_owned(), (choices).to_owned(), (visibility).to_owned(), (editable).to_owned(), (json!({"max_length":b.max_length.unwrap_or(4000)})).to_owned()).await?;audit(&mut tx,p.id,"profile_field.created","profile_field",id,Value::Null,json!({"key":b.key})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
