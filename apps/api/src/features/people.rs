use sea_orm::TransactionTrait;
use axum::{extract::{State,Path,Query},Json};
use serde::Deserialize;
use serde_json::{json,Value};
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::audit;
pub use crate::repositories::directory::Filter;
pub async fn list(State(s):State<AppState>,p:Principal,Query(f):Query<Filter>)->Result<Json<Value>>{
 let (rows,total)=crate::repositories::directory::list(&s.db,p.id,&f).await?;
 Ok(Json(json!({"items":rows,"total":total,"page":f.page.unwrap_or(1).clamp(1,100000),"page_size":f.page_size.unwrap_or(25).clamp(1,100)})))
}
pub async fn get(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{
 if id!=p.id{p.person(&s,id,"people.read").await?;}
 let mut user:Value=crate::repositories::people::get_select_people_directory(&s.db, (id).to_owned()).await?;
 user["managers"]=json!(crate::repositories::people::get_select_manager_relationships::<_,Value>(&s.db, (id).to_owned()).await?);
 let sensitive:bool=crate::repositories::people::get_select_policy(&s.db, (p.id).to_owned(), (id).to_owned()).await?;
 if id==p.id||sensitive{user["sensitive"]=crate::repositories::people::get_select_users(&s.db, (id).to_owned()).await?;}
 Ok(Json(user))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Update {pub display_name:Option<String>,pub status:Option<String>,pub volunteer_eligible:Option<bool>,pub personnel_level_id:Option<Uuid>,pub department_ids:Option<Vec<Uuid>>,pub primary_department_id:Option<Uuid>,pub manager_id:Option<Uuid>,pub clear_manager:Option<bool>}
pub async fn update(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Update>)->Result<Json<Value>>{
 p.person(&s,id,"people.edit").await?;
 if let Some(name)=&b.display_name{validation::text(name,120)?;}
 if b.status.as_deref().is_some_and(|v|!["active","inactive","former"].contains(&v)){return Err(ApiError::bad("Invalid status"));}
 // Cross-department membership and rank changes are global HR operations.
 if b.department_ids.is_some()||b.personnel_level_id.is_some()||b.manager_id.is_some()||b.primary_department_id.is_some()||b.clear_manager==Some(true){p.require(&s,"people.edit",None).await?;}
 let mut tx=s.db.begin().await?;
 crate::repositories::people::update_select_policy(&tx).await?; // serialize graph writes
 crate::repositories::people::update_select_users(&tx, (id).to_owned()).await?;
 let before:Value=crate::repositories::people::update_select_people_directory(&tx, (id).to_owned()).await?;
 if b.status.as_deref()==Some("active")&&before["status"]=="pending_identity"&&before["myfwa_id"].is_null(){return Err(ApiError::bad("Verify and link the external identity before activating this profile"));}
 crate::repositories::people::update_update_users(&tx, (id).to_owned(), (b.display_name).to_owned(), (b.status).to_owned(), (b.volunteer_eligible).to_owned(), (b.personnel_level_id).to_owned()).await?;
 if let Some(deps)=b.department_ids {
  if deps.len()>20||b.primary_department_id.is_some_and(|d|!deps.contains(&d)){return Err(ApiError::bad("Primary department must be an active membership"));}
  crate::repositories::people::update_update_user_departments(&tx, (id).to_owned()).await?;
  for dep in deps {crate::repositories::people::update_insert_user_departments(&tx, (id).to_owned(), (dep).to_owned(), (Some(dep)==b.primary_department_id).to_owned()).await?;}
  crate::repositories::people::update_update_users_2(&tx, (id).to_owned(), (b.primary_department_id).to_owned()).await?;
 }else if b.primary_department_id.is_some(){return Err(ApiError::bad("Include department_ids when setting a primary department"));}
 if b.clear_manager==Some(true){crate::repositories::people::update_update_manager_relationships(&tx, (id).to_owned()).await?;}
 if let Some(manager)=b.manager_id {
  let cycle:bool=crate::repositories::people::update_with_manager_relationships(&tx, (id).to_owned(), (manager).to_owned()).await?;
  if cycle{return Err(ApiError::bad("A manager relationship cannot create a cycle"));}
  crate::repositories::people::update_update_manager_relationships_2(&tx, (id).to_owned()).await?;
  crate::repositories::people::update_insert_manager_relationships(&tx, (manager).to_owned(), (id).to_owned()).await?;
 }
 audit(&mut tx,p.id,"people.updated","user",id,before,json!({"changed":true})).await?;tx.commit().await?;get(State(s),p,Path(id)).await
}
pub async fn directs(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>> {if id!=p.id{p.person(&s,id,"people.read").await?;}let rows:Vec<Value>=crate::repositories::people::directs_select_people_directory(&s.db, (id).to_owned(), (p.id).to_owned()).await?;Ok(Json(json!(rows)))}
pub async fn years(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>> {if id!=p.id{p.person(&s,id,"people.read").await?;}Ok(Json(json!(crate::repositories::people::years_select_user_service_years::<_,Value>(&s.db, (id).to_owned()).await?)))}
#[derive(Deserialize)]pub struct YearBody{pub verification_status:String}
pub async fn set_year(State(s):State<AppState>,p:Principal,Path((id,year)):Path<(Uuid,i32)>,Json(b):Json<YearBody>)->Result<Json<Value>> {
 p.person(&s,id,"people.edit").await?;validation::years(year)?;if !["verified","self_reported"].contains(&b.verification_status.as_str()){return Err(ApiError::bad("Invalid verification status"));}
 let mut tx=s.db.begin().await?;crate::repositories::people::set_year_insert_user_service_years(&tx, (id).to_owned(), (year).to_owned(), (&b.verification_status).to_owned(), (p.id).to_owned()).await?;crate::repositories::people::set_year_update_users(&tx, (id).to_owned()).await?;audit(&mut tx,p.id,"service_year.recorded","user",id,Value::Null,json!({"year":year,"verification_status":b.verification_status})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
pub async fn delete_year(State(s):State<AppState>,p:Principal,Path((id,year)):Path<(Uuid,i32)>)->Result<Json<Value>> {p.person(&s,id,"people.edit").await?;let mut tx=s.db.begin().await?;crate::repositories::people::delete_year_delete_user_service_years(&tx, (id).to_owned(), (year).to_owned()).await?;crate::repositories::people::delete_year_update_users(&tx, (id).to_owned()).await?;audit(&mut tx,p.id,"service_year.removed","user",id,json!({"year":year}),Value::Null).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
pub async fn skills(State(s):State<AppState>,_:Principal)->Result<Json<Value>> {Ok(Json(json!(crate::repositories::people::skills_select_skills::<_,Value>(&s.db).await?)))}
#[derive(Deserialize)]pub struct Name {pub name:String}
pub async fn create_skill(State(s):State<AppState>,p:Principal,Json(b):Json<Name>)->Result<Json<Value>> {p.require(&s,"people.edit",None).await?;validation::text(&b.name,80)?;let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::people::create_skill_insert_skills(&tx, (b.name.trim()).to_owned()).await?;audit(&mut tx,p.id,"skill.created","skill",id,Value::Null,json!({"name":b.name})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
pub async fn set_skill(State(s):State<AppState>,p:Principal,Path((id,skill)):Path<(Uuid,Uuid)>)->Result<Json<Value>> {change_skill(s,p,id,skill,true).await}
pub async fn remove_skill(State(s):State<AppState>,p:Principal,Path((id,skill)):Path<(Uuid,Uuid)>)->Result<Json<Value>> {change_skill(s,p,id,skill,false).await}
async fn change_skill(s:AppState,p:Principal,id:Uuid,skill:Uuid,add:bool)->Result<Json<Value>> {p.person(&s,id,"people.edit").await?;let mut tx=s.db.begin().await?;if add{crate::repositories::people::change_skill_insert_user_skills(&tx, (id).to_owned(), (skill).to_owned()).await?;}else{crate::repositories::people::change_skill_delete_user_skills(&tx, (id).to_owned(), (skill).to_owned()).await?;}crate::repositories::people::change_skill_update_users(&tx, (id).to_owned()).await?;audit(&mut tx,p.id,"skill.assignment_changed","user",id,Value::Null,json!({"skill_id":skill,"added":add})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
