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
pub async fn document()->Json<utoipa::openapi::OpenApi>{Json(HealthApi::openapi())}
