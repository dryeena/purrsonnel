use super::*;
use purrsonnel_api::features::request_follow_up::process_due;

async fn request(h:&Harness,hr:&Session)->Uuid {
 let(status,sent,_)=h.request("POST","/requests",Some(hr),request_body(Some(h.id("staff").await),None)).await;
 assert_eq!(status,StatusCode::OK,"{sent}");Uuid::parse_str(sent["id"].as_str().unwrap()).unwrap()
}
fn prefs(mode:&str)->Value{json!({"request_alerts":mode,"quiet_enabled":false,"timezone":"America/Denver","quiet_start":1320,"quiet_end":480})}
async fn due(h:&Harness,id:Uuid,escalate:Option<Uuid>){
 sqlx::query("UPDATE service_requests SET created_at=now()-interval '1 hour' WHERE id=$1").bind(id).execute(&h.db).await.unwrap();
 sqlx::query("INSERT INTO request_follow_ups(request_id,reminder_at,escalation_at,escalation_user_id) VALUES($1,now()-interval '2 minutes',CASE WHEN $2::uuid IS NOT NULL THEN now()-interval '1 minute' END,$2)").bind(id).bind(escalate).execute(&h.db).await.unwrap();
}
fn state(h:&Harness)->AppState{AppState::new(sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(h.db.clone()),Config{dev_auth:true,origin:"http://localhost:5173".into(),bind:"127.0.0.1:3000".into(),default_user:"dev.hr@purrsonnel.local".into(),max_volunteer_minutes:960})}

#[sqlx::test(migrations="./migrations")]
async fn alert_preferences_are_private_validated_and_do_not_remove_inbox(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;
 assert_eq!(h.request("GET","/notification-preferences",None,Value::Null).await.0,StatusCode::UNAUTHORIZED);
 let(_,defaults,_)=h.request("GET","/notification-preferences",Some(&hr),Value::Null).await;assert_eq!(defaults["request_alerts"],"all");
 for body in [prefs("arbitrary"),{let mut b=prefs("off");b["timezone"]=json!("not/a/timezone");b},{let mut b=prefs("off");b["quiet_end"]=json!(1320);b}]{assert_eq!(h.request("PUT","/notification-preferences",Some(&hr),body).await.0,StatusCode::BAD_REQUEST);}
 assert_eq!(h.request("PUT","/notification-preferences",Some(&hr),prefs("off")).await.0,StatusCode::OK);
 let id=request(&h,&hr).await;let staff=h.switch(&hr,"staff").await;
 let(_,own,_)=h.request("GET","/notification-preferences",Some(&staff),Value::Null).await;assert_eq!(own["request_alerts"],"all");
 assert_eq!(h.request("GET",&format!("/requests/{id}"),Some(&staff),Value::Null).await.0,StatusCode::OK);
}

#[sqlx::test(migrations="./migrations")]
async fn phone_policy_handles_overnight_daytime_boundaries_and_dst(db:PgPool){
 let h=Harness::new(db).await;let user=h.id("staff").await;
 sqlx::query("INSERT INTO notification_preferences(user_id,quiet_enabled,timezone) VALUES($1,true,'America/Denver')").bind(user).execute(&h.db).await.unwrap();
 for (at,expected) in [("2026-11-09T06:00:00Z","2026-11-09T15:00:00Z"),("2026-11-09T15:00:00Z","2026-11-09T15:00:00Z"),("2026-11-09T04:59:00Z","2026-11-09T04:59:00Z"),("2026-03-08T06:00:00Z","2026-03-08T14:00:00Z")]{
  let actual:chrono::DateTime<chrono::Utc>=sqlx::query_scalar("SELECT request_phone_after($1,'urgent',$2::text::timestamptz)").bind(user).bind(at).fetch_one(&h.db).await.unwrap();assert_eq!(actual,chrono::DateTime::parse_from_rfc3339(expected).unwrap());
 }
 sqlx::query("UPDATE notification_preferences SET quiet_start=540,quiet_end=1020 WHERE user_id=$1").bind(user).execute(&h.db).await.unwrap();
 let daytime:chrono::DateTime<chrono::Utc>=sqlx::query_scalar("SELECT request_phone_after($1,'normal','2026-11-09T18:00:00Z')").bind(user).fetch_one(&h.db).await.unwrap();assert_eq!(daytime.to_rfc3339(),"2026-11-10T00:00:00+00:00");
 sqlx::query("UPDATE notification_preferences SET request_alerts='urgent' WHERE user_id=$1").bind(user).execute(&h.db).await.unwrap();
 let suppressed:Option<chrono::DateTime<chrono::Utc>>=sqlx::query_scalar("SELECT request_phone_after($1,'normal',now())").bind(user).fetch_one(&h.db).await.unwrap();assert!(suppressed.is_none());
}

#[sqlx::test(migrations="./migrations")]
async fn follow_up_access_is_scoped_and_only_sender_can_schedule(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;
 let valid=json!({"reminder_at":chrono::Utc::now()+chrono::Duration::minutes(30)});
 assert_eq!(h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&hr),valid.clone()).await.0,StatusCode::OK);
 assert_eq!(h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&hr),json!({"reminder_after_minutes":15})).await.0,StatusCode::OK);
 let staff=h.switch(&hr,"staff").await;assert_eq!(h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&staff),valid).await.0,StatusCode::FORBIDDEN);
 let outsider=h.switch(&staff,"volunteer2").await;assert_eq!(h.request("GET",&format!("/requests/{id}/follow-up"),Some(&outsider),Value::Null).await.0,StatusCode::NOT_FOUND);
 let hr=h.login().await;let now=chrono::Utc::now();
 for invalid in [json!({"reminder_at":now}),json!({"escalation_at":now+chrono::Duration::hours(1)}),json!({"reminder_at":now+chrono::Duration::hours(2),"escalation_at":now+chrono::Duration::hours(1),"escalation_user_id":h.id("lead").await})]{assert_eq!(h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&hr),invalid).await.0,StatusCode::BAD_REQUEST);}
}

#[sqlx::test(migrations="./migrations")]
async fn manual_reminders_enforce_cooldown_and_follow_only_assignee(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let(_,sent,_)=h.request("POST","/requests",Some(&hr),request_body(None,Some(h.dept("Registration").await))).await;let id=sent["id"].as_str().unwrap();
 assert_eq!(h.request("POST",&format!("/requests/{id}/remind"),Some(&hr),json!({})).await.0,StatusCode::CONFLICT);
 let staff=h.switch(&hr,"staff").await;assert_eq!(h.request("POST",&format!("/requests/{id}/respond"),Some(&staff),json!({"action":"accept"})).await.0,StatusCode::OK);
 assert_eq!(h.request("POST",&format!("/requests/{id}/remind"),Some(&staff),json!({})).await.0,StatusCode::FORBIDDEN);
 sqlx::query("UPDATE service_requests SET created_at=now()-interval '1 hour' WHERE id=$1::text::uuid").bind(id).execute(&h.db).await.unwrap();let hr=h.login().await;
 let(status,result,_)=h.request("POST",&format!("/requests/{id}/remind"),Some(&hr),json!({})).await;assert_eq!(status,StatusCode::OK);assert_eq!(result["recipient_count"],1);
 assert_eq!(h.request("POST",&format!("/requests/{id}/remind"),Some(&hr),json!({})).await.0,StatusCode::CONFLICT);
 let notified:Vec<Uuid>=sqlx::query_scalar("SELECT recipient_user_id FROM notifications WHERE type='request_reminder'").fetch_all(&h.db).await.unwrap();assert_eq!(notified,vec![h.id("staff").await]);
}

#[sqlx::test(migrations="./migrations")]
async fn scheduled_steps_commit_once_under_competing_workers_without_push(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;let target=h.id("volunteer2").await;due(&h,id,Some(target)).await;
 let s=state(&h);let(a,b)=tokio::join!(process_due(&s),process_due(&s));a.unwrap();b.unwrap();process_due(&s).await.unwrap();
 for kind in ["request_reminder","request_escalated"]{let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type=$1").bind(kind).fetch_one(&h.db).await.unwrap();assert_eq!(count,1);}
 let recipient:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM request_recipients WHERE request_id=$1 AND user_id=$2)").bind(id).bind(target).fetch_one(&h.db).await.unwrap();assert!(recipient);
 let target_session=h.switch(&hr,"volunteer2").await;assert_eq!(h.request("POST",&format!("/requests/{id}/respond"),Some(&target_session),json!({"action":"accept"})).await.0,StatusCode::OK);
 assert!(sqlx::query("DELETE FROM request_activity WHERE request_id=$1").bind(id).execute(&h.db).await.is_err());
}

#[sqlx::test(migrations="./migrations")]
async fn completed_work_skips_all_scheduled_notifications(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;due(&h,id,Some(h.id("volunteer2").await)).await;
 sqlx::query("UPDATE service_requests SET status='completed',completed_at=now() WHERE id=$1").bind(id).execute(&h.db).await.unwrap();process_due(&state(&h)).await.unwrap();
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type IN ('request_reminder','request_escalated')").fetch_one(&h.db).await.unwrap();assert_eq!(count,0);
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM request_activity WHERE request_id=$1 AND kind IN ('reminder_skipped','escalation_skipped')").bind(id).fetch_one(&h.db).await.unwrap();assert_eq!(count,2);
}

#[sqlx::test(migrations="./migrations")]
async fn escalation_rechecks_revoked_sender_scope_and_retains_original_audience(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let lead=h.switch(&hr,"lead").await;let id=request(&h,&lead).await;let target=h.id("volunteer").await;
 let(status,result,_)=h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&lead),json!({"escalation_at":chrono::Utc::now()+chrono::Duration::hours(1),"escalation_user_id":target})).await;assert_eq!(status,StatusCode::OK,"{result}");
 sqlx::query("DELETE FROM permission_grants WHERE user_id=$1 AND permission='requests.send'").bind(h.id("lead").await).execute(&h.db).await.unwrap();
 sqlx::query("UPDATE request_follow_ups SET escalation_at=now()-interval '1 minute' WHERE request_id=$1").bind(id).execute(&h.db).await.unwrap();process_due(&state(&h)).await.unwrap();
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM request_recipients WHERE request_id=$1").bind(id).fetch_one(&h.db).await.unwrap();assert_eq!(count,1);
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type='request_escalated'").fetch_one(&h.db).await.unwrap();assert_eq!(count,0);
}

#[sqlx::test(migrations="./migrations")]
async fn policy_suppresses_phone_jobs_but_preserves_notifications(db:PgPool){
 use purrsonnel_api::features::request_follow_up::notify;
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;let staff=h.id("staff").await;
 sqlx::query("INSERT INTO notification_preferences(user_id,request_alerts) VALUES($1,'off')").bind(staff).execute(&h.db).await.unwrap();
 sqlx::query("INSERT INTO push_subscriptions(user_id,endpoint_hash,endpoint,p256dh,auth) VALUES($1,'fixture','https://fcm.googleapis.com/test','fixture','fixture')").bind(staff).execute(&h.db).await.unwrap();
 let mut tx=h.db.begin().await.unwrap();assert_eq!(notify(&mut tx,true,staff,id,"request_reminder","urgent").await.unwrap(),(0,1));tx.commit().await.unwrap();
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type='request_reminder'").fetch_one(&h.db).await.unwrap();assert_eq!(count,1);
 let staff_session=h.switch(&hr,"staff").await;assert_eq!(h.request("PUT","/notification-preferences",Some(&staff_session),prefs("all")).await.0,StatusCode::OK);
 let status:String=sqlx::query_scalar("SELECT status FROM push_deliveries LIMIT 1").fetch_one(&h.db).await.unwrap();assert_eq!(status,"suppressed");
 let(_,detail,_)=h.request("GET",&format!("/requests/{id}/follow-up"),Some(&staff_session),Value::Null).await;assert!(detail["delivery"].is_null());
}

#[sqlx::test(migrations="./migrations")]
async fn cancelling_a_request_notifies_other_team_members(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;
 assert_eq!(h.request("POST",&format!("/requests/{id}/respond"),Some(&hr),json!({"action":"cancel"})).await.0,StatusCode::OK);
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type='request_update' AND recipient_user_id=$1").bind(h.id("staff").await).fetch_one(&h.db).await.unwrap();assert_eq!(count,1);
 assert_eq!(h.request("PUT",&format!("/requests/{id}/follow-up"),Some(&hr),json!({"reminder_at":chrono::Utc::now()+chrono::Duration::hours(1)})).await.0,StatusCode::CONFLICT);
}

#[sqlx::test(migrations="./migrations")]
async fn quiet_hours_defer_jobs_without_consuming_attempts_and_preference_changes_wake_them(db:PgPool){
 use purrsonnel_api::features::request_follow_up::notify;
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;let staff=h.id("staff").await;
 sqlx::query("INSERT INTO notification_preferences(user_id,quiet_enabled,timezone,quiet_start,quiet_end) SELECT $1,true,'UTC',((extract(hour FROM now() AT TIME ZONE 'UTC')::int*60+extract(minute FROM now() AT TIME ZONE 'UTC')::int+1439)%1440)::smallint,((extract(hour FROM now() AT TIME ZONE 'UTC')::int*60+extract(minute FROM now() AT TIME ZONE 'UTC')::int+120)%1440)::smallint").bind(staff).execute(&h.db).await.unwrap();
 sqlx::query("INSERT INTO push_subscriptions(user_id,endpoint_hash,endpoint,p256dh,auth) VALUES($1,'fixture','https://fcm.googleapis.com/test','fixture','fixture')").bind(staff).execute(&h.db).await.unwrap();
 let mut tx=h.db.begin().await.unwrap();assert_eq!(notify(&mut tx,true,staff,id,"request_reminder","urgent").await.unwrap(),(1,0));tx.commit().await.unwrap();
 let delayed:bool=sqlx::query_scalar("SELECT next_attempt_at>now() AND attempts=0 AND status='pending' AND last_error='quiet_hours' FROM push_deliveries LIMIT 1").fetch_one(&h.db).await.unwrap();assert!(delayed);
 let staff_session=h.switch(&hr,"staff").await;assert_eq!(h.request("PUT","/notification-preferences",Some(&staff_session),prefs("all")).await.0,StatusCode::OK);
 let awake:bool=sqlx::query_scalar("SELECT next_attempt_at<=now() AND attempts=0 AND status='pending' FROM push_deliveries LIMIT 1").fetch_one(&h.db).await.unwrap();assert!(awake);
}

#[sqlx::test(migrations="./migrations")]
async fn removing_a_locked_schedule_prevents_later_scheduler_delivery(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;due(&h,id,None).await;
 let mut tx=h.db.begin().await.unwrap();sqlx::query("SELECT id FROM service_requests WHERE id=$1 FOR UPDATE").bind(id).execute(&mut *tx).await.unwrap();
 sqlx::query("UPDATE request_follow_ups SET reminder_at=NULL WHERE request_id=$1").bind(id).execute(&mut *tx).await.unwrap();
 process_due(&state(&h)).await.unwrap();tx.commit().await.unwrap();process_due(&state(&h)).await.unwrap();
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type='request_reminder'").fetch_one(&h.db).await.unwrap();assert_eq!(count,0);
}

#[sqlx::test(migrations="./migrations")]
async fn inactive_audiences_do_not_produce_false_reminder_success(db:PgPool){
 let h=Harness::new(db).await;let hr=h.login().await;let id=request(&h,&hr).await;due(&h,id,None).await;
 sqlx::query("UPDATE users SET status='inactive' WHERE id=$1").bind(h.id("staff").await).execute(&h.db).await.unwrap();
 assert_eq!(h.request("POST",&format!("/requests/{id}/remind"),Some(&hr),json!({})).await.0,StatusCode::CONFLICT);
 process_due(&state(&h)).await.unwrap();
 let kind:String=sqlx::query_scalar("SELECT kind FROM request_activity WHERE request_id=$1 AND kind<>'sent'").bind(id).fetch_one(&h.db).await.unwrap();assert_eq!(kind,"reminder_skipped");
 let count:i64=sqlx::query_scalar("SELECT count(*) FROM notifications WHERE type='request_reminder'").fetch_one(&h.db).await.unwrap();assert_eq!(count,0);
}
