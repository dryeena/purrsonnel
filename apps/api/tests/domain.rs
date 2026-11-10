use purrsonnel_api::{config::Config,features::lists::{validate_filter,matches,validate_operation},validation,auth::{hash,token},features::volunteers::{Entry,duration}};
use serde_json::json;
use chrono::{TimeZone,Utc};
use uuid::Uuid;

#[test]fn development_requires_explicit_opt_in(){assert!(Config::validate("development",true,"http://localhost:5173",false).is_ok());assert!(Config::validate("development",false,"http://localhost:5173",false).is_err());}
#[test]fn production_and_staging_never_allow_dev_auth(){for env in ["production","staging","test",""]{assert!(Config::validate(env,true,"http://localhost:5173",false).is_err());assert!(Config::validate(env,false,"http://localhost:5173",false).is_err());}}
#[test]fn dev_origins_must_be_loopback(){for origin in ["https://evil.example:5173","http://localhost.evil:5173","http://localhost:5173@evil.com","http://localhost:5173/path","null","http://localhost:"]{assert!(Config::validate("development",true,origin,false).is_err(),"{origin}");}for origin in ["http://127.0.0.1:5173","https://localhost:5173"]{assert!(Config::validate("development",true,origin,false).is_ok());}}
#[test]fn tokens_are_random_and_stored_as_hashes(){let a=token();let b=token();assert_eq!(a.len(),64);assert_ne!(a,b);assert_ne!(hash(&a),a);assert_eq!(hash("hello"),"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");}
#[test]fn free_text_is_bounded(){assert!(validation::text("  ",20).is_err());assert!(validation::text("hello",4).is_err());assert!(validation::text("🐈",1).is_ok());}
#[test]fn denial_requires_explanation(){assert!(validation::decision("deny","").is_err());assert!(validation::decision("deny","Not enough availability").is_ok());assert!(validation::decision("approve","").is_ok());assert!(validation::decision("accept","").is_err());}
#[test]fn explicit_years_are_bounded(){for year in [1989,2201]{assert!(validation::years(year).is_err());}for year in [2021,2023,2025]{assert!(validation::years(year).is_ok());}}
#[test]fn nested_filters_match_only_whitelisted_fields(){let f=json!({"op":"and","args":[{"op":"eq","field":"status","value":"active"},{"op":"or","args":[{"op":"contains","field":"display_name","value":"MAPLE"},{"op":"range","field":"verified_years","min":3,"max":5}]}]});assert!(validate_filter(&f,0).is_ok());assert!(matches(&f,&json!({"status":"active","display_name":"Maple Bunny","verified_years":0})));assert!(matches(&f,&json!({"status":"active","display_name":"Clover Bear","verified_years":3})));assert!(!matches(&f,&json!({"status":"former","display_name":"Maple Bunny","verified_years":5})));}
#[test]fn filters_cannot_reach_sensitive_fields_or_sql(){for field in ["legal_name","email","permission","password","status; DROP TABLE users;--"]{assert!(validate_filter(&json!({"op":"eq","field":field,"value":"x"}),0).is_err());}assert!(validate_filter(&json!({"op":"execute","sql":"SELECT 1"}),0).is_err());}
#[test]fn filters_enforce_complexity_limits(){let mut f=json!({"op":"eq","field":"status","value":"active"});for _ in 0..10{f=json!({"op":"not","arg":f});}assert!(validate_filter(&f,0).is_err());assert!(validate_filter(&json!({"op":"and","args":vec![json!({"op":"and","args":[]});21]}),0).is_err());}
#[test]fn filter_not_in_and_null_semantics(){let row=json!({"status":"active","department":null});assert!(matches(&json!({"op":"not","arg":{"op":"eq","field":"status","value":"former"}}),&row));assert!(matches(&json!({"op":"in","field":"status","value":["active","former"]}),&row));assert!(matches(&json!({"op":"is_null","field":"department"}),&row));assert!(matches(&json!({"op":"and","args":[]}),&row));assert!(!matches(&json!({"op":"or","args":[]}),&row));}
#[test]fn bulk_never_escalates_permissions(){for field in ["role","permissions","settings.manage","personnel_level_id","myfwa_id","legal_name"]{assert!(validate_operation(&json!({"field":field,"value":"system_admin"})).is_err());}assert!(validate_operation(&json!({"field":"status","value":"active","permission":"settings.manage"})).is_err());assert!(validate_operation(&json!({"field":"volunteer_eligible","value":"true"})).is_err());assert!(validate_operation(&json!({"field":"volunteer_eligible","value":true})).is_ok());}
#[test]fn schemas_reject_scripts_unknown_types_and_duplicate_keys(){for schema in [json!([{"key":"x","label":"X","type":"html"}]),json!([{"key":"<script>","label":"X","type":"text"}]),json!([{"key":"x","label":"X","type":"text"},{"key":"x","label":"Y","type":"number"}]),json!([{"key":"x","label":"X","type":"single_select","options":[]}])]{assert!(validation::validate_schema(&schema).is_err());}}
#[test]fn answers_preserve_types_requirements_and_choices(){let schema=json!([{"key":"name","label":"Name","type":"text","required":true,"max_length":10},{"key":"number","label":"Number","type":"number"},{"key":"choices","label":"Choices","type":"multi_select","options":["A","B"]}]);assert!(validation::validate_answers(&schema,&json!({"name":"Maple","number":2,"choices":["A"]})).is_ok());for answers in [json!({}),json!({"name":"Maple","number":"2"}),json!({"name":"Maple","choices":["C"]}),json!({"name":"Maple","unknown":true}),json!({"name":"Far too long to fit"})]{assert!(validation::validate_answers(&schema,&answers).is_err());}}
#[test]fn answers_reject_unsafe_links_and_invalid_dates(){let schema=json!([{"key":"url","type":"url"},{"key":"day","type":"date"}]);assert!(validation::validate_answers(&schema,&json!({"url":"javascript:alert(1)"})).is_err());assert!(validation::validate_answers(&schema,&json!({"day":"2027-02-30"})).is_err());assert!(validation::validate_answers(&schema,&json!({"url":"https://example.org","day":"2027-02-28"})).is_ok());}
fn entry()->Entry{Entry{department_id:Uuid::new_v4(),event_edition_id:None,work_date:Utc::now().date_naive(),minutes_worked:Some(90),started_at:None,ended_at:None,work_description:"Badge preparation".into()}}
#[test]fn volunteer_duration_accepts_exactly_one_input_mode(){let mut e=entry();assert_eq!(duration(&e).unwrap(),90);e.started_at=Some(Utc.with_ymd_and_hms(2027,5,7,9,0,0).unwrap());e.ended_at=Some(Utc.with_ymd_and_hms(2027,5,7,10,30,0).unwrap());assert!(duration(&e).is_err());e.minutes_worked=None;assert_eq!(duration(&e).unwrap(),90);e.ended_at=None;assert!(duration(&e).is_err());}
#[test]fn volunteer_duration_rejects_zero_negative_and_excessive(){let mut e=entry();for n in [-1,0,961,100000]{e.minutes_worked=Some(n);assert!(duration(&e).is_err());}e.minutes_worked=Some(960);assert_eq!(duration(&e).unwrap(),960);}

#[test]fn push_endpoints_cannot_be_used_for_internal_requests(){
 use purrsonnel_api::features::push::validate_endpoint;
 for endpoint in ["http://localhost:3000/admin","https://127.0.0.1/api","https://fcm.googleapis.com.attacker.example/push","https://user:pass@fcm.googleapis.com/push","https://fcm.googleapis.com:444/push","https://fcm.googleapis.com/push#secret"]{assert!(validate_endpoint(endpoint).is_err(),"{endpoint}");}
 for endpoint in ["https://fcm.googleapis.com/push/test","https://web.push.apple.com/test","https://updates.push.services.mozilla.com/wpush/v2/test","https://wns-test.notify.windows.com/test"]{assert!(validate_endpoint(endpoint).is_ok(),"{endpoint}");}
}
#[test]fn request_creation_requires_one_audience_and_bounded_content(){
 use purrsonnel_api::features::requests::{Create,validate_create};
 let mut b=Create{recipient_user_id:Some(Uuid::new_v4()),recipient_department_id:None,title:"Help at registration".into(),body:"Please bring badge stock to desk A".into(),urgency:"normal".into(),due_at:None,idempotency_key:Uuid::new_v4()};
 assert!(validate_create(&b).is_ok());b.recipient_department_id=Some(Uuid::new_v4());assert!(validate_create(&b).is_err());b.recipient_user_id=None;assert!(validate_create(&b).is_ok());b.body=" ".into();assert!(validate_create(&b).is_err());b.body="a".repeat(4001);assert!(validate_create(&b).is_err());
}
#[test]fn subscription_keys_must_be_real_sized_browser_key_material(){
 use purrsonnel_api::features::push::{Subscription,Keys,validate_subscription};use base64::{Engine,engine::general_purpose::URL_SAFE_NO_PAD};
 let mut public=vec![1;65];public[0]=4;let mut b=Subscription{endpoint:"https://fcm.googleapis.com/test".into(),keys:Keys{p256dh:URL_SAFE_NO_PAD.encode(&public),auth:URL_SAFE_NO_PAD.encode([1;16])}};
 assert!(validate_subscription(&b).is_ok());b.keys.auth="not-a-key".into();assert!(validate_subscription(&b).is_err());
}
#[test]fn follow_up_schedule_requires_a_bounded_future_and_ordered_escalation(){
 use purrsonnel_api::features::request_follow_up::{Schedule,validate_schedule};
 let now=Utc::now();let mut b=Schedule{reminder_at:Some(now+chrono::Duration::minutes(30)),escalation_at:None,escalation_user_id:None};assert!(validate_schedule(&b,now).is_ok());
 b.reminder_at=Some(now+chrono::Duration::minutes(14));assert!(validate_schedule(&b,now).is_err());b.reminder_at=Some(now+chrono::Duration::days(31));assert!(validate_schedule(&b,now).is_err());
 b.reminder_at=Some(now+chrono::Duration::hours(1));b.escalation_at=Some(now+chrono::Duration::minutes(30));b.escalation_user_id=Some(Uuid::new_v4());assert!(validate_schedule(&b,now).is_err());
 b.escalation_at=Some(now+chrono::Duration::hours(2));assert!(validate_schedule(&b,now).is_ok());b.escalation_user_id=None;assert!(validate_schedule(&b,now).is_err());
}
#[test]fn phone_preferences_reject_unbounded_minutes_and_unknown_modes(){
 use purrsonnel_api::features::alert_preferences::{Preferences,validate};
 let mut b=Preferences{request_alerts:"urgent".into(),quiet_enabled:true,timezone:"America/Denver".into(),quiet_start:1320,quiet_end:480};assert!(validate(&b).is_ok());
 for minute in [-1,1440,i16::MAX]{b.quiet_start=minute;assert!(validate(&b).is_err());}b.quiet_start=480;assert!(validate(&b).is_err());b.quiet_start=1320;b.request_alerts="bypass_quiet_hours".into();assert!(validate(&b).is_err());
}
#[test]fn relative_follow_up_times_use_one_server_clock_and_cannot_overflow(){
 use purrsonnel_api::features::request_follow_up::{ScheduleInput,resolve_schedule};
 let now=Utc::now();let input=|minutes|ScheduleInput{reminder_at:None,escalation_at:None,escalation_user_id:None,reminder_after_minutes:Some(minutes),escalation_after_minutes:None};
 assert_eq!(resolve_schedule(input(15),now).unwrap().reminder_at,Some(now+chrono::Duration::minutes(15)));
 for minutes in [-1,14,43201,i64::MAX]{assert!(resolve_schedule(input(minutes),now).is_err());}
 let mut conflicting=input(30);conflicting.reminder_at=Some(now+chrono::Duration::hours(1));assert!(resolve_schedule(conflicting,now).is_err());
}
