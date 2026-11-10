use axum::{extract::{State,Path},Json};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::audit;
const FIELDS:&[&str]=&["display_name","status","volunteer_eligible","level","department","verified_years","myfwa_id"];
/// Typed whitelist, bounded recursion and input size. No field is interpolated into SQL.
pub fn validate_filter(v:&Value,depth:usize)->Result<()> {
 if depth>8||v.to_string().len()>16000{return Err(ApiError::bad("Filter is too complex"));}
 match v["op"].as_str().unwrap_or("") {
  "and"|"or"=>{let args=v["args"].as_array().ok_or_else(||ApiError::bad("Group requires args"))?;if args.len()>20{return Err(ApiError::bad("Too many filter branches"));}for a in args{validate_filter(a,depth+1)?;}},
  "not"=>validate_filter(&v["arg"],depth+1)?,
  "eq"|"in"|"contains"|"range"|"is_null"=>{
   let field=v["field"].as_str().unwrap_or("");let op=v["op"].as_str().unwrap_or("");if !FIELDS.contains(&field){return Err(ApiError::bad("Unsupported filter field"));}
   let valid=|value:&Value|match field{"volunteer_eligible"=>value.is_boolean(),"verified_years"=>value.as_i64().is_some_and(|n|(0..=500).contains(&n)),_=>value.as_str().is_some_and(|s|s.len()<=500)};
   if op=="eq"&&!valid(&v["value"]){return Err(ApiError::bad("Filter value does not match the field type"));}
   if op=="in"&&!v["value"].as_array().is_some_and(|a|a.len()<=100&&a.iter().all(valid)){return Err(ApiError::bad("in requires up to 100 values of the field type"));}
   if op=="contains"&&(["verified_years","volunteer_eligible"].contains(&field)||!v["value"].is_string()){return Err(ApiError::bad("contains requires a text field and text value"));}
   if op=="range"&&(field!="verified_years"||!v["min"].is_number()||!v["max"].is_number()||v["min"].as_f64()>v["max"].as_f64()){return Err(ApiError::bad("range requires ordered numeric service year bounds"));}
  },
  _=>return Err(ApiError::bad("Unsupported filter operation"))
 }Ok(())
}
pub fn matches(v:&Value,row:&Value)->bool {let field=&row[v["field"].as_str().unwrap_or("")];match v["op"].as_str().unwrap_or(""){
 "and"=>v["args"].as_array().is_some_and(|a|a.iter().all(|x|matches(x,row))),"or"=>v["args"].as_array().is_some_and(|a|a.iter().any(|x|matches(x,row))),"not"=>!matches(&v["arg"],row),"eq"=>field==&v["value"],"in"=>v["value"].as_array().is_some_and(|a|a.contains(field)),"contains"=>field.as_str().zip(v["value"].as_str()).is_some_and(|(a,b)|a.to_lowercase().contains(&b.to_lowercase())),"is_null"=>field.is_null(),"range"=>field.as_f64().is_some_and(|n|n>=v["min"].as_f64().unwrap_or(f64::MAX)&&n<=v["max"].as_f64().unwrap_or(f64::MIN)),_=>false}}
async fn targets(s:&AppState,p:&Principal,filter:&Value,permission:&str)->Result<Vec<Value>>{validate_filter(filter,0)?;let rows:Vec<Value>=crate::repositories::lists::targets_select_people_directory(&s.db, (p.id).to_owned(), (permission).to_owned()).await?;if rows.len()>1000{return Err(ApiError::bad("This release supports up to 1,000 scoped people per list run; narrow organization access or add database AST compilation before scaling"));}Ok(rows.into_iter().filter(|r|matches(filter,r)).collect())}
pub async fn list(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{p.require(&s,"people.read",None).await?;Ok(Json(json!(crate::repositories::lists::list_select_saved_lists::<_,Value>(&s.db, (p.id).to_owned()).await?)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Saved {pub name:String,pub filter_ast_json:Value}
pub async fn create(State(s):State<AppState>,p:Principal,Json(b):Json<Saved>)->Result<Json<Value>>{p.require(&s,"people.read",None).await?;validation::text(&b.name,120)?;validate_filter(&b.filter_ast_json,0)?;let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::lists::create_insert_saved_lists(&tx, (b.name).to_owned(), (p.id).to_owned(), (b.filter_ast_json).to_owned()).await?;audit(&mut tx,p.id,"list.created","saved_list",id,Value::Null,json!({"private":true})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))}
pub async fn run(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let filter:Value=crate::repositories::lists::run_select_saved_lists(&s.db, (id).to_owned(), (p.id).to_owned()).await?;let rows=targets(&s,&p,&filter,"people.read").await?;Ok(Json(json!({"total":rows.len(),"items":rows})))}
pub async fn delete(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{crate::repositories::lists::delete_delete_saved_lists(&s.db, (id).to_owned(), (p.id).to_owned()).await?;Ok(Json(json!({"ok":true})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Preview {pub filter:Value,pub operation:Value}
pub fn validate_operation(op:&Value)->Result<()> {
 if !op.is_object()||op.as_object().unwrap().keys().any(|k|!["field","value"].contains(&k.as_str())){return Err(ApiError::bad("Invalid bulk operation"));}
 match op["field"].as_str().unwrap_or("") {"status"=>{if !["active","inactive","former"].contains(&op["value"].as_str().unwrap_or("")){return Err(ApiError::bad("Invalid personnel status"));}},"volunteer_eligible"=>{if !op["value"].is_boolean(){return Err(ApiError::bad("Expected a boolean"));}},_=>return Err(ApiError::bad("Bulk supports status and volunteer eligibility only; permission grants cannot be bulk edited"))}Ok(())
}
pub async fn preview(State(s):State<AppState>,p:Principal,Json(b):Json<Preview>)->Result<Json<Value>>{p.require(&s,"people.bulk_update",None).await?;validate_operation(&b.operation)?;let rows=targets(&s,&p,&b.filter,"people.edit").await?;if rows.len()>250{return Err(ApiError::bad("Narrow this update to at most 250 people"));}let snapshots:Vec<Value>=rows.iter().map(|r|json!({"id":r["id"],"updated_at":r["updated_at"],"before":r[b.operation["field"].as_str().unwrap_or("")],"display_name":r["display_name"]})).collect();let id:Uuid=crate::repositories::lists::preview_insert_bulk_change_jobs(&s.db, (p.id).to_owned(), (b.filter).to_owned(), (json!(snapshots)).to_owned(), (&b.operation).to_owned(), (rows.len() as i32).to_owned()).await?;Ok(Json(json!({"id":id,"count":rows.len(),"targets":snapshots,"operation":b.operation,"expires_in_minutes":30})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Execute {pub preview_id:Uuid,pub confirmed:bool}
pub async fn execute(State(s):State<AppState>,p:Principal,Json(b):Json<Execute>)->Result<Json<Value>>{
 p.require(&s,"people.bulk_update",None).await?;if !b.confirmed{return Err(ApiError::bad("Explicit confirmation is required"));}let mut tx=s.db.begin().await?;
 let job=crate::repositories::lists::execute_select_bulk_change_jobs(&tx, (b.preview_id).to_owned(), (p.id).to_owned()).await?;
 if job.try_get::<String>("", "status")?!="previewed"{return Err(ApiError::conflict("This preview has already been applied"));}if job.try_get::<chrono::DateTime<chrono::Utc>>("", "created_at")?<chrono::Utc::now()-chrono::Duration::minutes(30){return Err(ApiError::conflict("Preview expired. Review a fresh preview."));}
 let op:Value=job.try_get("", "operation_json")?;validate_operation(&op)?;let snapshots:Value=job.try_get("", "target_user_ids_json")?;let mut results=vec![];
 for target in snapshots.as_array().ok_or_else(||ApiError::bad("Invalid stored snapshot"))? {
  let id=Uuid::parse_str(target["id"].as_str().unwrap_or("")).map_err(|_|ApiError::bad("Invalid snapshot"))?;
  let allowed:bool=crate::repositories::lists::execute_select_policy(&tx, (p.id).to_owned(), (id).to_owned()).await?;
  let current:Option<Value>=crate::repositories::lists::execute_select_users(&tx, (id).to_owned()).await?;
  if !allowed||current.as_ref()!=Some(&target["updated_at"]){results.push(json!({"id":id,"status":"conflict"}));continue;}
  if op["field"]=="status"{crate::repositories::lists::execute_update_users(&tx, (id).to_owned(), (op["value"].as_str().unwrap()).to_owned()).await?;}else{crate::repositories::lists::execute_update_users_2(&tx, (id).to_owned(), (op["value"].as_bool().unwrap()).to_owned()).await?;}
  audit(&mut tx,p.id,"people.bulk_updated","user",id,json!({"field":op["field"],"value":target["before"]}),op.clone()).await?;results.push(json!({"id":id,"status":"updated"}));
 }
 let status=if results.iter().any(|r|r["status"]=="conflict"){"partially_failed"}else{"completed"};crate::repositories::lists::execute_update_bulk_change_jobs(&tx, (b.preview_id).to_owned(), (status).to_owned(), (json!(results)).to_owned()).await?;tx.commit().await?;Ok(Json(json!({"status":status,"results":results})))
}
pub async fn jobs(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{p.require(&s,"people.bulk_update",None).await?;Ok(Json(json!(crate::repositories::lists::jobs_select_bulk_change_jobs::<_,Value>(&s.db, (p.id).to_owned()).await?)))}
