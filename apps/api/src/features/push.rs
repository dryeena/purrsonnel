//! Web Push transport and a durable PostgreSQL outbox. No external broker.
//! Payloads deliberately contain no request text, names, or personnel data.
use axum::{extract::State,Json};
use base64::{Engine,engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use serde_json::{json,Value};
use sea_orm::TransactionTrait;
use std::{sync::Arc,time::Duration};
use uuid::Uuid;
use web_push::{SubscriptionInfo,VapidSignatureBuilder,WebPushMessageBuilder,ContentEncoding};
use crate::{AppState,auth::{Principal,hash},error::{ApiError,Result}};

pub struct PushService {public_key:String,private_key:String,subject:String,client:reqwest::Client}
impl PushService {
 pub fn from_env()->std::result::Result<Option<Arc<Self>>,String> {
  if std::env::var("WEB_PUSH_ENABLED").as_deref()!=Ok("true"){return Ok(None);}
  let public_key=std::env::var("VAPID_PUBLIC_KEY").map_err(|_|"VAPID_PUBLIC_KEY is required")?;
  let private_key=std::env::var("VAPID_PRIVATE_KEY").map_err(|_|"VAPID_PRIVATE_KEY is required")?;
  let subject=std::env::var("VAPID_SUBJECT").map_err(|_|"VAPID_SUBJECT is required")?;
  Ok(Some(Self::configured(public_key,private_key,subject)?))
 }
 pub fn configured(public_key:String,private_key:String,subject:String)->std::result::Result<Arc<Self>,String> {
  if !subject.starts_with("mailto:")&&!subject.starts_with("https://"){return Err("VAPID_SUBJECT must be an operator contact mailto: or HTTPS URL".into());}
  let public=URL_SAFE_NO_PAD.decode(&public_key).map_err(|_|"Invalid VAPID public key")?;
  let private=URL_SAFE_NO_PAD.decode(&private_key).map_err(|_|"Invalid VAPID private key")?;
  if public.len()!=65||public[0]!=4||private.len()!=32{return Err("VAPID requires a P-256 key pair (65-byte public / 32-byte private)".into());}
  let parsed=VapidSignatureBuilder::from_base64_no_sub(&private_key).map_err(|_|"Invalid VAPID signing key")?;
  if parsed.get_public_key().as_slice()!=public.as_slice(){return Err("VAPID public and private keys do not match".into());}
  let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(12)).build().map_err(|_|"Unable to initialize push transport")?;
  Ok(Arc::new(Self{public_key,private_key,subject,client}))
 }
 async fn send(&self,info:&SubscriptionInfo,payload:&[u8])->std::result::Result<u16,()> {
  validate_endpoint(&info.endpoint).map_err(|_|())?;
  let mut signature=VapidSignatureBuilder::from_base64(&self.private_key,info).map_err(|_|())?;
  signature.add_claim("sub",self.subject.as_str());
  let mut builder=WebPushMessageBuilder::new(info);
  builder.set_vapid_signature(signature.build().map_err(|_|())?);
  builder.set_payload(ContentEncoding::Aes128Gcm,payload);
  builder.set_ttl(3600);
  let message=builder.build().map_err(|_|())?;
  let encrypted=message.payload.ok_or(())?;
  let mut request=self.client.post(&info.endpoint).header("ttl",message.ttl.to_string()).header("urgency","high").header("content-type","application/octet-stream").header("content-encoding","aes128gcm");
  for (key,value) in encrypted.crypto_headers{request=request.header(key,value);}
  Ok(request.body(encrypted.content).send().await.map_err(|_|())?.status().as_u16())
 }
}

pub fn validate_endpoint(endpoint:&str)->Result<()> {
 let url=reqwest::Url::parse(endpoint).map_err(|_|ApiError::bad("Invalid push endpoint"))?;
 let host=url.host_str().unwrap_or("");
 let allowed=["fcm.googleapis.com","updates.push.services.mozilla.com","web.push.apple.com"].contains(&host)||host.ends_with(".notify.windows.com");
 if endpoint.len()>4096||!allowed||url.scheme()!="https"||!url.username().is_empty()||url.password().is_some()||url.port_or_known_default()!=Some(443)||url.fragment().is_some(){return Err(ApiError::bad("Unsupported browser push service. Use a supported browser or contact your administrator."));}
 Ok(())
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Keys{pub p256dh:String,pub auth:String}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Subscription{pub endpoint:String,pub keys:Keys}
pub fn validate_subscription(b:&Subscription)->Result<()> {
 validate_endpoint(&b.endpoint)?;
 let public=URL_SAFE_NO_PAD.decode(&b.keys.p256dh).map_err(|_|ApiError::bad("Invalid device encryption key"))?;
 let auth=URL_SAFE_NO_PAD.decode(&b.keys.auth).map_err(|_|ApiError::bad("Invalid device authentication key"))?;
 if public.len()!=65||public[0]!=4||auth.len()!=16{return Err(ApiError::bad("Invalid device encryption keys"));}Ok(())
}
pub async fn config(State(s):State<AppState>,_:Principal)->Json<Value>{Json(json!({"enabled":s.push.is_some(),"public_key":s.push.as_ref().map(|p|p.public_key.as_str())}))}
pub async fn subscribe(State(s):State<AppState>,p:Principal,Json(b):Json<Subscription>)->Result<Json<Value>> {
 if s.push.is_none(){return Err(ApiError(axum::http::StatusCode::SERVICE_UNAVAILABLE,"Phone alerts have not been configured on this server. Requests still appear in your inbox.".into()));}
 validate_subscription(&b)?;
 let mut tx=s.db.begin().await?;
 // A device cannot be silently reassigned from a different identity.
 let id:Option<Uuid>=crate::repositories::push::subscribe_insert_push_subscriptions(&tx, (p.id).to_owned(), (hash(&b.endpoint)).to_owned(), (&b.endpoint).to_owned(), (&b.keys.p256dh).to_owned(), (&b.keys.auth).to_owned()).await?;
 let id=id.ok_or_else(||ApiError::conflict("This device is linked to another person. Turn alerts off in that account before enabling them here."))?;
 // Inactive endpoint takeover is explicit after client unsubscribe. Clear old queued
 // deliveries and set ownership atomically, never deliver old-user notifications.
 crate::repositories::push::subscribe_update_push_subscriptions(&tx, (id).to_owned(), (p.id).to_owned()).await?;
 crate::repositories::push::subscribe_update_push_deliveries(&tx, (id).to_owned(), (p.id).to_owned()).await?;
 super::audit(&mut tx,p.id,"push.device_enabled","push_subscription",id,Value::Null,json!({"enabled":true})).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Endpoint{pub endpoint:String}
pub async fn subscription_status(State(s):State<AppState>,p:Principal,Json(b):Json<Endpoint>)->Result<Json<Value>> {
 let active:bool=crate::repositories::push::subscription_status_select_push_subscriptions(&s.db, (hash(&b.endpoint)).to_owned(), (p.id).to_owned()).await?;
 Ok(Json(json!({"active":active&&s.push.is_some()})))
}
pub async fn unsubscribe(State(s):State<AppState>,p:Principal,Json(b):Json<Endpoint>)->Result<Json<Value>> {
 let mut tx=s.db.begin().await?;
 crate::repositories::push::unsubscribe_update_push_subscriptions(&tx, (hash(&b.endpoint)).to_owned(), (p.id).to_owned()).await?;
 crate::repositories::push::unsubscribe_update_push_deliveries(&tx, (hash(&b.endpoint)).to_owned(), (p.id).to_owned()).await?;tx.commit().await?;Ok(Json(json!({"ok":true})))
}

pub async fn run(s:AppState){
 if s.push.is_none(){return;}
 loop {
  // Four independent leased jobs per cycle, with bounded parallel network calls.
  let results=tokio::join!(deliver_one(&s),deliver_one(&s),deliver_one(&s),deliver_one(&s));
  for result in [results.0,results.1,results.2,results.3]{if let Err(error)=result{tracing::error!(status=?error.0,"push outbox cycle failed");}}
  tokio::time::sleep(Duration::from_secs(2)).await;
 }
}
async fn deliver_one(s:&AppState)->Result<()> {
 let Some(service)=&s.push else{return Ok(());};
 let mut tx=s.db.begin().await?;
 let job:Option<Uuid>=crate::repositories::push::deliver_one_select_push_deliveries(&tx).await?;
 let Some(id)=job else{return Ok(());};
 let row=crate::repositories::push::deliver_one_select_push_deliveries_2(&tx, (id).to_owned()).await?;
 let now=chrono::Utc::now();let allowed:Option<chrono::DateTime<chrono::Utc>>=row.try_get("", "allowed_after")?;
 let expired=!row.try_get::<bool>("", "active")?||!row.try_get::<bool>("", "same_owner")?||!row.try_get::<bool>("", "active_user")?||row.try_get::<bool>("", "obsolete")?||row.try_get::<chrono::DateTime<chrono::Utc>>("", "expires_at")?<=now;
 if expired||allowed.is_none(){
  crate::repositories::push::deliver_one_update_push_deliveries(&tx, (id).to_owned(), (if expired{"expired"}else{"suppressed"}).to_owned(), (if expired{"expired_or_obsolete"}else{"preferences"}).to_owned()).await?;
  tx.commit().await?;return Ok(());
 }
 if allowed.is_some_and(|at|at>now){
  crate::repositories::push::deliver_one_update_push_deliveries_2(&tx, (id).to_owned(), (allowed).to_owned()).await?;
  tx.commit().await?;return Ok(());
 }
 let lease=Uuid::new_v4();
 crate::repositories::push::deliver_one_update_push_deliveries_3(&tx, (id).to_owned(), (lease).to_owned()).await?;
 tx.commit().await?;
 let payload:Value=row.try_get("", "payload_json")?;let notification:Uuid=row.try_get("", "notification_id")?;
 let safe_payload=json!({"kind":row.try_get::<String>("", "notification_type")?,"url":format!("/requests?id={}",payload["request_id"].as_str().unwrap_or("")),"tag":format!("request-{notification}")}).to_string();
 let info=SubscriptionInfo::new(row.try_get::<String>("", "endpoint")?,row.try_get::<String>("", "p256dh")?,row.try_get::<String>("", "auth")?);
 let result=service.send(&info,safe_payload.as_bytes()).await;
 let (status,error)=match result{Ok(200..=299)=>("sent",None),Ok(404|410)=>("expired",Some("subscription_expired")),_ if row.try_get::<i32>("", "attempts")?+1>=5=>("failed",Some("delivery_failed")),_=>("pending",Some("retry_scheduled"))};
 let mut tx=s.db.begin().await?;
 let changed=crate::repositories::push::deliver_one_update_push_deliveries_4(&tx, (id).to_owned(), (status).to_owned(), (error).to_owned(), (lease).to_owned()).await?.rows_affected();
 if status=="expired"&&changed>0{crate::repositories::push::deliver_one_update_push_subscriptions(&tx, (row.try_get::<Uuid>("", "subscription_id")?).to_owned()).await?;}
 tx.commit().await?;Ok(())
}
