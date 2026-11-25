use axum::{extract::State,Json};
use serde::Serialize;
use utoipa::{OpenApi,ToSchema};
use crate::{AppState,error::Result};
#[derive(Serialize,ToSchema)]pub struct Health {status:String}
#[utoipa::path(get,path="/health/live",responses((status=200,description="Process is running",body=Health)))]
pub async fn live()->Json<Health>{Json(Health{status:"live".into()})}
#[utoipa::path(get,path="/health/ready",responses((status=200,description="Database is reachable",body=Health),(status=500,description="Database unavailable")))]
pub async fn ready(State(s):State<AppState>)->Result<Json<Health>>{crate::repositories::openapi::ready_select_policy(&s.db).await?;Ok(Json(Health{status:"ready".into()}))}
#[derive(OpenApi)]#[openapi(info(title="Purrsonnel health API",version="0.1.0",description="Health schema. See docs/API.md for the full authenticated feature API contract."),paths(live,ready),components(schemas(Health)))]
struct HealthApi;
pub async fn document()->Json<serde_json::Value>{
 let mut document=serde_json::to_value(HealthApi::openapi()).expect("OpenAPI is serializable");
 document["info"]["title"]=serde_json::json!("Purrsonnel API");
 document["info"]["description"]=serde_json::json!("Authenticated endpoints use the current browser session. For mutations, obtain csrf_token from /api/v1/auth/me and provide it in X-CSRF-Token. Development bootstrap is available only when enabled. See docs/API.md for field-level constraints.");
 document["components"]["securitySchemes"]=serde_json::json!({"session":{"type":"apiKey","in":"cookie","name":"purrsonnel_session"},"csrf":{"type":"apiKey","in":"header","name":"X-CSRF-Token"}});
 let endpoints=[
  ("/api/v1/auth/me","get","Current session"),("/api/v1/dashboard","get","Dashboard"),
  ("/api/v1/users","get","Search people"),("/api/v1/users/{id}","get","Person details"),("/api/v1/users/{id}","patch","Edit person"),
  ("/api/v1/organization/tree","get","Organization catalog"),("/api/v1/events","get","Event editions"),("/api/v1/events","post","Create event"),
  ("/api/v1/schedules","get","List shifts"),("/api/v1/schedules","post","Create shift"),("/api/v1/schedules/staff","get","Find staff for assignment"),("/api/v1/schedules/{id}/claim","post","Claim opening"),("/api/v1/schedules/{id}/assignments","post","Assign staff"),
  ("/api/v1/requests","get","List requests"),("/api/v1/requests","post","Send request"),("/api/v1/requests/{id}","get","Request details"),
  ("/api/v1/positions","get","List positions"),("/api/v1/applications","get","List applications"),("/api/v1/interviews","get","List interviews"),
  ("/api/v1/terms/applicable","get","Applicable policies"),("/api/v1/volunteer/entries","get","Volunteer entries"),("/api/v1/volunteer/reports/hours","get","Volunteer hours report"),
  ("/api/v1/permission-thresholds","get","Permission minimum levels"),("/api/v1/permission-thresholds","put","Set permission minimum level"),
 ];
 for (path,method,summary) in endpoints {
  let mut operation=serde_json::json!({"summary":summary,"security":[{"session":[]}],"responses":{"200":{"description":"Successful response"},"400":{"description":"Invalid request"},"401":{"description":"Authentication required"},"403":{"description":"Permission denied"}}});
  if path.contains("{id}"){operation["parameters"]=serde_json::json!([{"name":"id","in":"path","required":true,"schema":{"type":"string","format":"uuid"}}]);}
  if method!="get"{operation["security"]=serde_json::json!([{"session":[],"csrf":[]}]);operation["requestBody"]=serde_json::json!({"required":true,"content":{"application/json":{"schema":{"type":"object","additionalProperties":true}}}});}
  document["paths"][path][method]=operation;
 }
 document["paths"]["/api/v1/schedules"]["post"]["requestBody"]=serde_json::json!({"required":true,"content":{"application/json":{"schema":{"type":"object","required":["event_edition_id","department_id","title","starts_at","ends_at","slots"],"properties":{"event_edition_id":{"type":"string","format":"uuid"},"department_id":{"type":"string","format":"uuid"},"title":{"type":"string","maxLength":160},"starts_at":{"type":"string","format":"date-time"},"ends_at":{"type":"string","format":"date-time"},"location_label":{"type":"string"},"slots":{"type":"integer","minimum":1,"maximum":200}}}}}});
 document["paths"]["/api/v1/schedules/{id}/assignments"]["post"]["requestBody"]=serde_json::json!({"required":true,"content":{"application/json":{"schema":{"type":"object","required":["user_id"],"properties":{"user_id":{"type":"string","format":"uuid"}}}}}});
 document["paths"]["/api/v1/permission-thresholds"]["put"]["requestBody"]=serde_json::json!({"required":true,"content":{"application/json":{"schema":{"type":"object","required":["permission","minimum_rank"],"properties":{"permission":{"type":"string"},"minimum_rank":{"type":["integer","null"],"minimum":1,"maximum":1000}}}}}});
 document["paths"]["/api/v1/schedules/{id}/claim"]["post"].as_object_mut().unwrap().remove("requestBody");
 document["paths"]["/api/v1/schedules"]["get"]["parameters"]=serde_json::json!([{"name":"event_id","in":"query","required":false,"schema":{"type":"string","format":"uuid"}}]);
 document["paths"]["/api/v1/schedules/staff"]["get"]["parameters"]=serde_json::json!([{"name":"department_id","in":"query","required":true,"description":"Department whose schedule the caller may edit","schema":{"type":"string","format":"uuid"}}]);
 Json(document)
}
