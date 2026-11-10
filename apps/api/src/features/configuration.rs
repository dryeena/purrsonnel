use axum::{extract::{State,Path},Json};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::audit;
pub const PERMISSIONS:&[&str]=&["requests.send","people.read","people.edit","people.bulk_update","people.read_sensitive","organization.manage","terms.manage","terms.read_compliance","volunteer.time.submit","volunteer.time.approve","volunteer.time.admin","positions.manage","applications.submit","applications.review","interviews.manage","interviews.decide","interviews.finalize","settings.manage","audit.read"];
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Grant {pub user_id:Uuid,pub permission:String,pub scope_type:String,pub scope_id:Option<Uuid>}
pub async fn grants(State(s):State<AppState>,p:Principal)->Result<Json<Value>> {p.require(&s,"settings.manage",None).await?;Ok(Json(json!({"permissions":PERMISSIONS,"items":crate::repositories::configuration::grants_select_permission_grants::<_,Value>(&s.db).await?})))}
pub async fn grant(State(s):State<AppState>,p:Principal,Json(b):Json<Grant>)->Result<Json<Value>>{
 p.require(&s,"settings.manage",None).await?;if !PERMISSIONS.contains(&b.permission.as_str())||!["global","department","division"].contains(&b.scope_type.as_str())||(b.scope_type=="global")!=b.scope_id.is_none(){return Err(ApiError::bad("Invalid permission or scope"));}
 if let Some(id)=b.scope_id{let exists:bool=if b.scope_type=="department"{crate::repositories::configuration::grant_select_departments(&s.db, (id).to_owned()).await?}else{crate::repositories::configuration::grant_select_divisions(&s.db, (id).to_owned()).await?};if !exists{return Err(ApiError::bad("Scope not found"));}}
 let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::configuration::grant_insert_permission_grants(&tx, (b.user_id).to_owned(), (&b.permission).to_owned(), (&b.scope_type).to_owned(), (b.scope_id).to_owned(), (p.id).to_owned()).await?;
 audit(&mut tx,p.id,"permission.granted","permission_grant",id,Value::Null,json!({"user_id":b.user_id,"permission":b.permission,"scope_type":b.scope_type,"scope_id":b.scope_id})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
pub async fn revoke(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;let mut tx=s.db.begin().await?;crate::repositories::configuration::revoke_select_policy(&tx).await?;let row=crate::repositories::configuration::revoke_select_permission_grants(&tx, (id).to_owned()).await?;
 if row.try_get::<String>("", "permission")?=="settings.manage"&&row.try_get::<String>("", "scope_type")?=="global"{let count:i64=crate::repositories::configuration::revoke_select_permission_grants_2(&tx, (id).to_owned()).await?;if count==0{return Err(ApiError::conflict("Cannot remove the last active system administrator"));}}
 crate::repositories::configuration::revoke_delete_permission_grants(&tx, (id).to_owned()).await?;audit(&mut tx,p.id,"permission.revoked","permission_grant",id,row.try_get("", "data")?,Value::Null).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
pub async fn profile(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{
 if id!=p.id{p.person(&s,id,"people.read").await?;}
 let sensitive:bool=crate::repositories::configuration::profile_select_policy(&s.db, (p.id).to_owned(), (id).to_owned()).await?;
 let rows:Vec<Value>=crate::repositories::configuration::profile_select_profile_field_definitions(&s.db, (id).to_owned(), (sensitive).to_owned(), (id==p.id).to_owned()).await?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Profile {pub values:std::collections::HashMap<Uuid,Value>}
pub async fn update_profile(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Profile>)->Result<Json<Value>>{if id!=p.id{p.person(&s,id,"people.edit").await?;}if b.values.len()>50{return Err(ApiError::bad("Too many fields"));}let mut tx=s.db.begin().await?;
 for (field,value) in b.values{
 let row=crate::repositories::configuration::update_profile_select_profile_field_definitions(&tx, (field).to_owned()).await?;
 if id!=p.id||row.try_get::<String>("", "editable_by")?!="self"{p.person(&s,id,"people.edit").await?;}
 if id!=p.id&&row.try_get::<String>("", "visibility")?=="sensitive"{p.person(&s,id,"people.read_sensitive").await?;}
 let key:String=row.try_get("", "key")?;let rules:Value=row.try_get("", "validation_json")?;let schema=json!([{"key":key,"label":row.try_get::<String>("", "label")?,"type":row.try_get::<String>("", "type")?,"options":row.try_get::<Value>("", "choices_json")?,"required":false,"max_length":rules["max_length"].as_u64().unwrap_or(4000)}]);let mut answers=json!({});answers[&key]=value.clone();validation::validate_answers(&schema,&answers)?;
 crate::repositories::configuration::update_profile_insert_profile_field_values(&tx, (id).to_owned(), (field).to_owned(), (value).to_owned(), (p.id).to_owned()).await?;
 audit(&mut tx,p.id,"profile_field.value_updated","user",id,Value::Null,json!({"field_id":field})).await?;}
 crate::repositories::configuration::update_profile_update_users(&tx, (id).to_owned()).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Level {pub label:String,pub slug:String,pub rank:i32}
pub async fn create_level(State(s):State<AppState>,p:Principal,Json(b):Json<Level>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;validation::text(&b.label,80)?;validation::text(&b.slug,80)?;if !(1..=1000).contains(&b.rank){return Err(ApiError::bad("Rank must be 1–1000"));}let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::configuration::create_level_insert_personnel_levels(&tx, (&b.label).to_owned(), (b.slug).to_owned(), (b.rank).to_owned()).await?;audit(&mut tx,p.id,"level.created","personnel_level",id,Value::Null,json!({"label":b.label,"rank":b.rank})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Mapping {pub personnel_level_id:Uuid}
pub async fn map_level(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Mapping>)->Result<Json<Value>>{p.require(&s,"settings.manage",None).await?;let mut tx=s.db.begin().await?;let old:Option<Uuid>=crate::repositories::configuration::map_level_select_position_levels(&tx, (id).to_owned()).await?;crate::repositories::configuration::map_level_update_position_levels(&tx, (id).to_owned(), (b.personnel_level_id).to_owned()).await?;audit(&mut tx,p.id,"position_level.mapped","position_level",id,json!({"personnel_level_id":old}),json!({"personnel_level_id":b.personnel_level_id})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
pub async fn templates(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{let rows:Vec<Value>=crate::repositories::configuration::templates_select_form_template_versions(&s.db, (p.id).to_owned()).await?;Ok(Json(json!(rows)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Template {pub name:String,pub department_id:Option<Uuid>,pub schema:Value}
pub async fn create_template(State(s):State<AppState>,p:Principal,Json(b):Json<Template>)->Result<Json<Value>>{
 p.require(&s,"positions.manage",b.department_id).await?;validation::text(&b.name,160)?;let mut tx=s.db.begin().await?;
 let mut fields:Vec<Value>=crate::repositories::configuration::create_template_select_organization_default_questions(&tx).await?;
 fields.extend(b.schema.as_array().ok_or_else(||ApiError::bad("Schema must be a question array"))?.clone());let schema=json!(fields);validation::validate_schema(&schema)?;
 let id:Uuid=crate::repositories::configuration::create_template_insert_form_templates(&tx, (b.name).to_owned(), (b.department_id).to_owned(), (p.id).to_owned()).await?;
 crate::repositories::configuration::create_template_insert_form_template_versions(&tx, (id).to_owned(), (schema).to_owned()).await?;audit(&mut tx,p.id,"form_template.created","form_template",id,Value::Null,json!({"version":1})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct TemplateVersion {pub schema:Value}
pub async fn template_version(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<TemplateVersion>)->Result<Json<Value>>{let mut tx=s.db.begin().await?;let dept:Option<Uuid>=crate::repositories::configuration::template_version_select_form_templates(&tx, (id).to_owned()).await?;p.require(&s,"positions.manage",dept).await?;validation::validate_schema(&b.schema)?;let version:Uuid=crate::repositories::configuration::template_version_insert_form_template_versions(&tx, (id).to_owned(), (b.schema).to_owned()).await?;audit(&mut tx,p.id,"form_template.versioned","form_template",id,Value::Null,json!({"version_id":version})).await?;tx.commit().await?;Ok(Json(json!({"id":version})))}
