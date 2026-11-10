use axum::{extract::{Path,Query,State},Json};
use chrono::{DateTime,Utc};
use sea_orm::{ConnectionTrait,DbBackend,Statement,TransactionTrait,TryGetable,Value};
use serde::Deserialize;
use serde_json::{json,Value as JsonValue};
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result},validation};
use super::audit;

fn stmt(sql:&str,values:Vec<Value>)->Statement{Statement::from_sql_and_values(DbBackend::Postgres,sql,values)}
#[derive(Deserialize)]pub struct Filter{event_id:Option<Uuid>}
pub async fn list(State(s):State<AppState>,p:Principal,Query(f):Query<Filter>)->Result<Json<JsonValue>>{
 let q="SELECT jsonb_build_object('id',x.id,'event_edition_id',x.event_edition_id,'event',e.name,'department_id',x.department_id,'department',d.name,'title',x.title,'starts_at',x.starts_at,'ends_at',x.ends_at,'location_label',x.location_label,'slots',x.slots,'assigned',COALESCE((SELECT jsonb_agg(jsonb_build_object('id',u.id,'display_name',u.display_name)) FROM event_shift_assignments a JOIN users u ON u.id=a.user_id WHERE a.shift_id=x.id),'[]'),'assigned_count',(SELECT count(*) FROM event_shift_assignments a WHERE a.shift_id=x.id),'can_manage',has_permission($1,'people.edit',x.department_id),'claimed_by_me',EXISTS(SELECT 1 FROM event_shift_assignments a WHERE a.shift_id=x.id AND a.user_id=$1)) FROM event_shifts x JOIN event_editions e ON e.id=x.event_edition_id JOIN departments d ON d.id=x.department_id WHERE ($2::uuid IS NULL OR x.event_edition_id=$2) AND (has_permission($1,'people.read',x.department_id) OR EXISTS(SELECT 1 FROM event_shift_assignments a WHERE a.shift_id=x.id AND a.user_id=$1) OR (e.active AND (SELECT volunteer_eligible FROM users WHERE id=$1))) ORDER BY x.starts_at,d.name";
 let rows:Vec<JsonValue>=s.db.query_all(stmt(q,vec![p.id.into(),f.event_id.into()])).await?.into_iter().map(|r|r.try_get_by_index(0)).collect::<std::result::Result<_,_>>()?;Ok(Json(json!(rows)))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Shift{event_edition_id:Uuid,department_id:Uuid,title:String,starts_at:DateTime<Utc>,ends_at:DateTime<Utc>,location_label:Option<String>,slots:i32}
pub async fn create(State(s):State<AppState>,p:Principal,Json(b):Json<Shift>)->Result<Json<JsonValue>>{
 p.require(&s,"people.edit",Some(b.department_id)).await?;validation::text(&b.title,160)?;
 if b.ends_at<=b.starts_at||b.slots<1||b.slots>200{return Err(ApiError::bad("Provide a valid shift time and number of openings"));}
 let mut tx=s.db.begin().await?;
 let id:Uuid=tx.query_one(stmt("INSERT INTO event_shifts(event_edition_id,department_id,title,starts_at,ends_at,location_label,slots,created_by_user_id) SELECT $1,$2,$3,$4,$5,$6,$7,$8 WHERE EXISTS(SELECT 1 FROM event_editions WHERE id=$1 AND active) RETURNING id",vec![b.event_edition_id.into(),b.department_id.into(),b.title.into(),b.starts_at.into(),b.ends_at.into(),b.location_label.unwrap_or_default().into(),b.slots.into(),p.id.into()])).await?.ok_or_else(||ApiError::bad("Choose an active event"))?.try_get_by_index(0)?;
 audit(&mut tx,p.id,"schedule.shift_created","event_shift",id,JsonValue::Null,json!({"department_id":b.department_id})).await?;tx.commit().await?;Ok(Json(json!({"id":id})))
}
pub async fn claim(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<JsonValue>>{
 let n=s.db.execute(stmt("INSERT INTO event_shift_assignments(shift_id,user_id) SELECT x.id,$2 FROM event_shifts x JOIN event_editions e ON e.id=x.event_edition_id JOIN users u ON u.id=$2 WHERE x.id=$1 AND e.active AND u.volunteer_eligible AND (SELECT count(*) FROM event_shift_assignments WHERE shift_id=x.id)<x.slots ON CONFLICT DO NOTHING",vec![id.into(),p.id.into()])).await?.rows_affected();
 if n==0{return Err(ApiError::conflict("This opening is no longer available"));}Ok(Json(json!({"ok":true})))
}
