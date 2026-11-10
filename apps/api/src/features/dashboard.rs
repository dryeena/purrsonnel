use axum::{extract::{State,Query},Json};
use serde_json::{json,Value};
use crate::{AppState,auth::Principal,error::Result};
use super::Page;
pub async fn dashboard(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{
 let counts:Value=crate::repositories::dashboard::dashboard_select_service_requests(&s.db, (p.id).to_owned()).await?;
 let departments:Vec<Value>=crate::repositories::dashboard::dashboard_select_departments(&s.db, (p.id).to_owned()).await?;
 let activity:Vec<Value>=crate::repositories::dashboard::dashboard_select_notifications(&s.db, (p.id).to_owned()).await?;
 Ok(Json(json!({"counts":counts,"departments":departments,"notifications":activity})))
}
pub async fn audit(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{p.require(&s,"audit.read",None).await?;let rows:Vec<Value>=crate::repositories::dashboard::audit_select_audit_events(&s.db, (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))}
