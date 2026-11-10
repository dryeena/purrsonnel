use axum::{extract::{State,Path},Json};
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result}};
pub async fn departments(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{let Json(tree)=super::organization::tree(State(s),p).await?;Ok(Json(tree["departments"].clone()))}
pub async fn divisions(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{let Json(tree)=super::organization::tree(State(s),p).await?;Ok(Json(tree["divisions"].clone()))}
pub async fn levels(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{let Json(tree)=super::organization::tree(State(s),p).await?;Ok(Json(tree["levels"].clone()))}
pub async fn department(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let Json(tree)=super::organization::tree(State(s),p).await?;let found=tree["departments"].as_array().and_then(|rows|rows.iter().find(|d|d["id"]==id.to_string())).cloned().ok_or_else(ApiError::missing)?;Ok(Json(found))}
pub async fn entry(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let row=crate::repositories::detail::entry_select_volunteer_entries(&s.db, (id).to_owned()).await?;let department:Uuid=row.try_get("", "department_id")?;if row.try_get::<Uuid>("", "volunteer_user_id")?!=p.id&&!p.can(&s,"volunteer.time.approve",Some(department)).await?&&!p.can(&s,"volunteer.time.admin",Some(department)).await?{return Err(ApiError::forbidden());}Ok(Json(row.try_get("", "data")?))}
pub async fn position(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let row=crate::repositories::detail::position_select_position_postings(&s.db, (id).to_owned()).await?;if row.try_get::<String>("", "status")?!="open"{p.require(&s,"positions.manage",Some(row.try_get("", "department_id")?)).await?;}Ok(Json(row.try_get("", "data")?))}
pub async fn interview(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let row=crate::repositories::detail::interview_select_interview_participants(&s.db, (id).to_owned(), (p.id).to_owned()).await?;if row.try_get::<Uuid>("", "applicant_user_id")?!=p.id&&!row.try_get::<bool>("", "panel_member")?{p.require(&s,"interviews.manage",Some(row.try_get("", "department_id")?)).await?;}Ok(Json(row.try_get("", "data")?))}
pub async fn saved_list(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let data:Value=crate::repositories::detail::saved_list_select_saved_lists(&s.db, (id).to_owned(), (p.id).to_owned()).await?;Ok(Json(data))}
pub async fn export_person(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{
 p.require(&s,"people.read_sensitive",None).await?;p.person(&s,id,"people.read").await?;
 let person:Value=crate::repositories::detail::export_person_select_users(&s.db, (id).to_owned()).await?;
 let signatures:Vec<Value>=crate::repositories::detail::export_person_select_term_signatures(&s.db, (id).to_owned()).await?;
 let entries:Vec<Value>=crate::repositories::detail::export_person_select_volunteer_entries(&s.db, (id).to_owned()).await?;
 let applications:Vec<Value>=crate::repositories::detail::export_person_select_applications(&s.db, (id).to_owned()).await?;
 let directory:Value=crate::repositories::detail::export_person_select_people_directory(&s.db, (id).to_owned()).await?;
 let years:Vec<Value>=crate::repositories::detail::export_person_select_user_service_years(&s.db, (id).to_owned()).await?;
 let custom_fields:Vec<Value>=crate::repositories::detail::export_person_select_profile_field_values(&s.db, (id).to_owned()).await?;
 let mut tx=s.db.begin().await?;super::audit(&mut tx,p.id,"person.subject_exported","user",id,Value::Null,json!({"sensitive_export":true})).await?;tx.commit().await?;
 Ok(Json(json!({"person":person,"directory":directory,"service_years":years,"custom_fields":custom_fields,"terms_evidence":signatures,"volunteer_entries":entries,"applications":applications})))
}
