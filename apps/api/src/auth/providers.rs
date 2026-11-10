/// All providers resolve to the same immutable local user ID; permissions live in PostgreSQL.
#[allow(async_fn_in_trait)]
pub trait AuthProvider {
 fn name(&self)->&'static str;
 fn available(&self)->bool;
 /// Resolve a provider-validated subject to a stable local user. Production callers
 /// must validate the external protocol before invoking this boundary.
 async fn resolve(&self,db:&sea_orm::DatabaseConnection,subject:&str)->crate::error::Result<uuid::Uuid>;
}
pub struct DevAuthProvider;
impl AuthProvider for DevAuthProvider {
 fn name(&self)->&'static str{"development"}fn available(&self)->bool{true}
 async fn resolve(&self,db:&sea_orm::DatabaseConnection,subject:&str)->crate::error::Result<uuid::Uuid>{Ok(crate::repositories::providers::resolve_select_users(db, (subject).to_owned()).await?)}
}
/// Unimplemented by design: no myFWA protocol details have been supplied.
pub struct MyFwaOidcProvider {pub issuer:String,pub client_id:String,pub client_secret:String,pub redirect_uri:String,pub requested_scopes:Vec<String>,pub claims_mapping:serde_json::Value}
impl AuthProvider for MyFwaOidcProvider {
 fn name(&self)->&'static str{"myfwa_oidc"}fn available(&self)->bool{false}
 async fn resolve(&self,_db:&sea_orm::DatabaseConnection,_subject:&str)->crate::error::Result<uuid::Uuid>{Err(crate::error::ApiError(axum::http::StatusCode::NOT_IMPLEMENTED,"myFWA authentication is not configured or implemented".into()))}
}
