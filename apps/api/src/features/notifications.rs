use axum::{extract::{State,Path,Query},Json};
use serde_json::{json,Value};
use uuid::Uuid;
use crate::{AppState,auth::Principal,error::{ApiError,Result}};
use super::Page;
pub async fn list(State(s):State<AppState>,p:Principal,Query(page):Query<Page>)->Result<Json<Value>>{let rows:Vec<Value>=crate::repositories::notifications::list_select_notifications(&s.db, (p.id).to_owned(), (page.limit()).to_owned(), (page.offset()).to_owned()).await?;Ok(Json(json!(rows)))}
pub async fn read(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>)->Result<Json<Value>>{let affected=crate::repositories::notifications::read_update_notifications(&s.db, (id).to_owned(), (p.id).to_owned()).await?.rows_affected();if affected==0{return Err(ApiError::missing());}Ok(Json(json!({"ok":true})))}
