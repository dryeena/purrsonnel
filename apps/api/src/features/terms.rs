use sea_orm::TransactionTrait;
use axum::{extract::{State,Path},Json};
use serde::Deserialize;
use serde_json::{json,Value};
use uuid::Uuid;
use crate::{AppState,auth::{Principal,hash},error::{ApiError,Result},validation};
use super::audit;
pub async fn applicable(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{
 let rows:Vec<Value>=crate::repositories::terms::applicable_select_terms(&s.db, (p.id).to_owned()).await?;Ok(Json(json!(rows)))
}
pub async fn list(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{p.require(&s,"terms.manage",None).await?;let rows:Vec<Value>=crate::repositories::terms::list_select_term_versions(&s.db).await?;Ok(Json(json!(rows)))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Create {pub title:String,pub audience:String,pub body_markdown:String,pub minimum_personnel_rank:Option<i32>,pub requires_volunteer:Option<bool>}
pub async fn create(State(s):State<AppState>,p:Principal,Json(b):Json<Create>)->Result<Json<Value>>{p.require(&s,"terms.manage",None).await?;validation::text(&b.title,200)?;validation::text(&b.body_markdown,50000)?;if !["all","volunteers","personnel"].contains(&b.audience.as_str()){return Err(ApiError::bad("Invalid audience"));}let mut tx=s.db.begin().await?;let id:Uuid=crate::repositories::terms::create_insert_terms(&tx, (b.title).to_owned(), (b.audience).to_owned(), (b.minimum_personnel_rank).to_owned(), (b.requires_volunteer).to_owned(), (p.id).to_owned()).await?;let version:Uuid=crate::repositories::terms::create_insert_term_versions(&tx, (id).to_owned(), (&b.body_markdown).to_owned(), (hash(&b.body_markdown)).to_owned()).await?;audit(&mut tx,p.id,"term.drafted","term",id,Value::Null,json!({"version_id":version})).await?;tx.commit().await?;Ok(Json(json!({"id":id,"version_id":version})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Version {pub body_markdown:String}
pub async fn version(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Version>)->Result<Json<Value>>{p.require(&s,"terms.manage",None).await?;validation::text(&b.body_markdown,50000)?;let mut tx=s.db.begin().await?;crate::repositories::terms::version_select_terms(&tx, (id).to_owned()).await?;let v:Uuid=crate::repositories::terms::version_insert_term_versions(&tx, (id).to_owned(), (&b.body_markdown).to_owned(), (hash(&b.body_markdown)).to_owned()).await?;audit(&mut tx,p.id,"term.version_drafted","term",id,Value::Null,json!({"version_id":v})).await?;tx.commit().await?;Ok(Json(json!({"id":v})))}
#[derive(Deserialize)]pub struct Publish {pub version_id:Uuid}
pub async fn publish(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Publish>)->Result<Json<Value>>{p.require(&s,"terms.manage",None).await?;let mut tx=s.db.begin().await?;crate::repositories::terms::publish_select_terms(&tx, (id).to_owned()).await?;let draft:bool=crate::repositories::terms::publish_select_term_versions(&tx, (b.version_id).to_owned(), (id).to_owned()).await?;if !draft{return Err(ApiError::conflict("Only an unpublished draft may be published"));}crate::repositories::terms::publish_update_term_versions(&tx, (id).to_owned()).await?;crate::repositories::terms::publish_update_term_versions_2(&tx, (b.version_id).to_owned(), (p.id).to_owned()).await?;audit(&mut tx,p.id,"term.published","term",id,Value::Null,json!({"version_id":b.version_id})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Sign {pub typed_name:String,pub consent:bool}
pub async fn sign(State(s):State<AppState>,p:Principal,Path(id):Path<Uuid>,Json(b):Json<Sign>)->Result<Json<Value>>{validation::text(&b.typed_name,200)?;if !b.consent{return Err(ApiError::bad("Affirmative consent is required"));}let mut tx=s.db.begin().await?;
 let term:Uuid=crate::repositories::terms::sign_select_term_versions(&tx, (id).to_owned()).await?;
 crate::repositories::terms::sign_select_terms(&tx, (term).to_owned()).await?;
 let digest:String=crate::repositories::terms::sign_select_term_versions_2(&tx, (id).to_owned(), (p.id).to_owned()).await?.ok_or_else(||ApiError::conflict("This version is not currently applicable; reload the document"))?;
 let receipt:Value=crate::repositories::terms::sign_insert_term_signatures(&tx, (id).to_owned(), (p.id).to_owned(), (b.typed_name.trim()).to_owned(), (digest).to_owned()).await?.unwrap_or(Value::Null);
 if !receipt.is_null(){audit(&mut tx,p.id,"term.signed","term_version",id,Value::Null,json!({"method":"authenticated_clickwrap_typed_name"})).await?;}
 let receipt=if receipt.is_null(){crate::repositories::terms::sign_select_term_signatures(&tx, (id).to_owned(), (p.id).to_owned()).await?}else{receipt};tx.commit().await?;Ok(Json(receipt))
}
pub async fn compliance(State(s):State<AppState>,p:Principal)->Result<Json<Value>>{
 let rows:Vec<Value>=crate::repositories::terms::compliance_select_users(&s.db, (p.id).to_owned()).await?;Ok(Json(json!(rows)))
}
