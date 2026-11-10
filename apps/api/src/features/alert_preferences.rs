//! User-owned phone preferences. Never changes inbox delivery or request permissions.
use sea_orm::TransactionTrait;
use axum::{extract::State,Json};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::{AppState,auth::Principal,error::{ApiError,Result}};

#[derive(Deserialize,Serialize)]#[serde(deny_unknown_fields)]
pub struct Preferences {pub request_alerts:String,pub quiet_enabled:bool,pub timezone:String,pub quiet_start:i16,pub quiet_end:i16}
pub fn validate(b:&Preferences)->Result<()> {
 if !["all","urgent","off"].contains(&b.request_alerts.as_str()) || !(0..1440).contains(&b.quiet_start) || !(0..1440).contains(&b.quiet_end) || b.quiet_start==b.quiet_end || b.timezone.len()>100 {
  return Err(ApiError::bad("Choose an alert mode, timezone, and different quiet-hour start/end times"));
 }Ok(())
}
pub async fn get(State(s):State<AppState>,p:Principal)->Result<Json<Value>> {
 let value:Option<Value>=crate::repositories::alert_preferences::get_select_notification_preferences(&s.db, (p.id).to_owned()).await?;
 Ok(Json(value.unwrap_or(json!({"request_alerts":"all","quiet_enabled":false,"timezone":"UTC","quiet_start":1320,"quiet_end":480}))))
}
pub async fn put(State(s):State<AppState>,p:Principal,Json(b):Json<Preferences>)->Result<Json<Value>> {
 validate(&b)?;
 let timezone:bool=crate::repositories::alert_preferences::put_select_pg_timezone_names(&s.db, (&b.timezone).to_owned()).await?;
 if !timezone{return Err(ApiError::bad("Choose a recognized timezone, such as America/Denver"));}
 let mut tx=s.db.begin().await?;
 crate::repositories::alert_preferences::put_insert_notification_preferences(&tx, (p.id).to_owned(), (&b.request_alerts).to_owned(), (b.quiet_enabled).to_owned(), (&b.timezone).to_owned(), (b.quiet_start).to_owned(), (b.quiet_end).to_owned()).await?;
 // Wake deferred jobs for policy reevaluation. Suppressed alerts stay suppressed;
 // enabling alerts never floods the phone with previously opted-out notifications.
 crate::repositories::alert_preferences::put_update_push_deliveries(&tx, (p.id).to_owned()).await?;
 super::audit(&mut tx,p.id,"notifications.preferences_updated","user",p.id,Value::Null,json!(b)).await?;
 tx.commit().await?;Ok(Json(json!({"ok":true})))
}
