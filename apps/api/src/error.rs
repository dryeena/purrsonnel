use axum::{http::StatusCode, response::{IntoResponse,Response}, Json};
use serde_json::json;
pub type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
pub struct ApiError(pub StatusCode,pub String);
impl ApiError {
 pub fn bad(message: impl Into<String>) -> Self {Self(StatusCode::BAD_REQUEST,message.into())}
 pub fn forbidden() -> Self {Self(StatusCode::FORBIDDEN,"You do not have permission for this action".into())}
 pub fn conflict(message: impl Into<String>) -> Self {Self(StatusCode::CONFLICT,message.into())}
 pub fn missing() -> Self {Self(StatusCode::NOT_FOUND,"Record not found".into())}
 pub fn unauthorized() -> Self {Self(StatusCode::UNAUTHORIZED,"A development session is required".into())}
}
impl From<sea_orm::DbErr> for ApiError {
 fn from(e:sea_orm::DbErr)->Self {
  use sea_orm::{DbErr,SqlErr};
  if matches!(&e,DbErr::RecordNotFound(_)){return Self::missing();}
  if let DbErr::Exec(sea_orm::RuntimeErr::SqlxError(error))|DbErr::Query(sea_orm::RuntimeErr::SqlxError(error))=&e{
   if error.as_database_error().is_some_and(|db|db.code().as_deref()==Some("23514")){return Self::bad("Invalid value or related record");}
  }
  match e.sql_err(){
   Some(SqlErr::UniqueConstraintViolation(_))=>Self::conflict("This record or operation already exists"),
   Some(SqlErr::ForeignKeyConstraintViolation(_))=>Self::bad("Invalid value or related record"),
   _=>{tracing::error!("database operation failed");Self(StatusCode::INTERNAL_SERVER_ERROR,"The operation could not be completed".into())}
  }
 }
}
impl IntoResponse for ApiError {fn into_response(self)->Response {(self.0,Json(json!({"type":"about:blank","title":self.0.canonical_reason().unwrap_or("Error"),"status":self.0.as_u16(),"detail":self.1}))).into_response()}}
