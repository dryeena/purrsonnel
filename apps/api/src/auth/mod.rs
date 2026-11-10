use axum::{extract::{FromRequestParts,State}, http::{request::Parts,HeaderMap,HeaderValue}, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use uuid::Uuid;
use crate::{AppState,error::{ApiError,Result}};
pub mod providers;
use providers::{AuthProvider,DevAuthProvider};
pub fn token() -> String { use rand::RngCore; let mut bytes=[0u8;32];rand::rngs::OsRng.fill_bytes(&mut bytes);hex::encode(bytes) }
pub fn hash(value:&str)->String {use sha2::{Digest,Sha256};hex::encode(Sha256::digest(value.as_bytes()))}
pub fn cookie(headers:&HeaderMap)->Option<&str>{headers.get("cookie")?.to_str().ok()?.split(';').find_map(|part|part.trim().strip_prefix("purrsonnel_session="))}
#[derive(Clone,Debug)]
pub struct Principal {pub id:Uuid}
impl FromRequestParts<AppState> for Principal {
 type Rejection=ApiError;
 async fn from_request_parts(parts:&mut Parts,state:&AppState)->Result<Self>{
  let raw=cookie(&parts.headers).ok_or_else(ApiError::unauthorized)?;
  let id=crate::repositories::auth::from_request_parts_select_sessions(&state.db, (hash(raw)).to_owned()).await?.ok_or_else(ApiError::unauthorized)?;
  Ok(Self{id})
 }
}
impl Principal {
 pub async fn can(&self,s:&AppState,permission:&str,department:Option<Uuid>)->Result<bool>{Ok(crate::repositories::auth::can_select_policy(&s.db, (self.id).to_owned(), (permission).to_owned(), (department).to_owned()).await?)}
 pub async fn require(&self,s:&AppState,permission:&str,department:Option<Uuid>)->Result<()> {if self.can(s,permission,department).await?{Ok(())}else{Err(ApiError::forbidden())}}
 pub async fn person(&self,s:&AppState,id:Uuid,permission:&str)->Result<()> {let ok:bool=crate::repositories::auth::person_select_policy(&s.db, (self.id).to_owned(), (id).to_owned(), (permission).to_owned()).await?;if ok{Ok(())}else{Err(ApiError::forbidden())}}
}
pub async fn csrf(State(s):State<AppState>,req:axum::extract::Request,next:axum::middleware::Next)->Result<axum::response::Response>{
 if !req.uri().path().starts_with("/health/") {
  let expected=s.config.origin.split("://").nth(1).unwrap_or("");
  if req.headers().get("host").and_then(|h|h.to_str().ok())!=Some(expected){return Err(ApiError::forbidden());}
 }
 // Origin is mandatory on all browser mutations, including bootstrap. No CORS wildcard.
 if !matches!(*req.method(),axum::http::Method::GET|axum::http::Method::HEAD|axum::http::Method::OPTIONS){
  if req.headers().get("origin").and_then(|v|v.to_str().ok())!=Some(s.config.origin.as_str()) {return Err(ApiError::forbidden());}
  if req.headers().get("sec-fetch-site").and_then(|v|v.to_str().ok()).is_some_and(|v|v!="same-origin" && v!="none"){return Err(ApiError::forbidden());}
  {
   let key=if req.uri().path().ends_with("/auth/dev/bootstrap"){"bootstrap".into()}else{cookie(req.headers()).map(hash).unwrap_or_else(||"bootstrap".into())};
   let mut limits=s.mutation_limits.lock().map_err(|_|ApiError::bad("Please retry"))?;
   limits.retain(|_,(at,_)|at.elapsed().as_secs()<60);
   if limits.len()>10000{return Err(ApiError(axum::http::StatusCode::TOO_MANY_REQUESTS,"Please retry in a minute".into()));}
   let limit=if key=="bootstrap"{30}else{120};let entry=limits.entry(key).or_insert((std::time::Instant::now(),0));entry.1+=1;
   if entry.1>limit{return Err(ApiError(axum::http::StatusCode::TOO_MANY_REQUESTS,"Too many mutations. Please retry in a minute".into()));}
  }
  if !req.uri().path().ends_with("/auth/dev/bootstrap") {
   let raw=cookie(req.headers()).ok_or_else(ApiError::unauthorized)?;
   let csrf=req.headers().get("x-csrf-token").and_then(|v|v.to_str().ok()).ok_or_else(ApiError::forbidden)?;
   let valid:bool=crate::repositories::auth::csrf_select_sessions(&s.db, (hash(raw)).to_owned(), (csrf).to_owned()).await?;
   if !valid{return Err(ApiError::forbidden());}
  }
 }
 Ok(next.run(req).await)
}
async fn establish(s:&AppState,id:Uuid,old:Option<&str>)->Result<impl IntoResponse>{
 let raw=token();let csrf=token();let mut tx=s.db.begin().await?;
 if let Some(old)=old {crate::repositories::auth::establish_delete_sessions(&tx, (hash(old)).to_owned()).await?;}
 crate::repositories::auth::establish_delete_sessions_2(&tx).await?;
 crate::repositories::auth::establish_insert_sessions(&tx, (hash(&raw)).to_owned(), (id).to_owned(), (&csrf).to_owned()).await?;
 crate::repositories::auth::establish_update_users(&tx, (id).to_owned()).await?;
 crate::repositories::auth::establish_insert_audit_events(&tx, (id).to_owned()).await?;tx.commit().await?;
 let secure=if s.config.origin.starts_with("https:"){ "; Secure" }else{""};
 Ok(([(axum::http::header::SET_COOKIE,HeaderValue::from_str(&format!("purrsonnel_session={raw}; HttpOnly; SameSite=Lax; Path=/; Max-Age=43200{secure}")).unwrap())],Json(json!({"csrf_token":csrf}))))
}
pub async fn bootstrap(State(s):State<AppState>,headers:HeaderMap)->Result<axum::response::Response>{
 if !s.config.dev_auth{return Err(ApiError::missing());}
 if let Some(raw)=cookie(&headers){if let Some(csrf)=crate::repositories::auth::bootstrap_select_sessions::<_,String>(&s.db, (hash(raw)).to_owned()).await?{return Ok(Json(json!({"csrf_token":csrf})).into_response());}}
 let id=DevAuthProvider.resolve(&s.db,&s.config.default_user).await?;
 Ok(establish(&s,id,cookie(&headers)).await?.into_response())
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Switch {user_id:Uuid}
pub async fn switch(State(s):State<AppState>,_:Principal,headers:HeaderMap,Json(body):Json<Switch>)->Result<axum::response::Response>{
 if !s.config.dev_auth{return Err(ApiError::missing());}
 let id=crate::repositories::auth::switch_select_users(&s.db, (body.user_id).to_owned()).await?;
 Ok(establish(&s,id,cookie(&headers)).await?.into_response())
}
pub async fn me(State(s):State<AppState>,p:Principal,headers:HeaderMap)->Result<Json<Value>>{
 let user:Value=crate::repositories::auth::me_select_people_directory(&s.db, (p.id).to_owned()).await?;
 let grants:Vec<Value>=crate::repositories::auth::me_select_permission_grants(&s.db, (p.id).to_owned()).await?;
 let identities:Vec<Value>=if s.config.dev_auth{crate::repositories::auth::me_select_users(&s.db).await?}else{vec![]};
 let row=crate::repositories::auth::me_select_sessions(&s.db, (hash(cookie(&headers).unwrap_or(""))).to_owned()).await?;
 Ok(Json(json!({"user":user,"grants":grants,"dev_mode":s.config.dev_auth,"identities":identities,"csrf_token":row.try_get::<String>("", "csrf_token")?})))
}
pub async fn logout(State(s):State<AppState>,_:Principal,headers:HeaderMap)->Result<impl IntoResponse>{crate::repositories::auth::logout_delete_sessions(&s.db, (hash(cookie(&headers).unwrap_or(""))).to_owned()).await?;Ok(([(axum::http::header::SET_COOKIE,"purrsonnel_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")],Json(json!({"ok":true}))))}
