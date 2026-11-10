pub mod app;
pub mod auth;
pub mod config;
pub mod error;
pub mod features;
pub mod validation;
pub mod openapi;
pub mod entities;
pub mod repositories;
pub mod migrations;

use sea_orm::DatabaseConnection;
#[derive(Clone)]
pub struct AppState { pub db: DatabaseConnection, pub config: config::Config, pub mutation_limits: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String,(std::time::Instant,u32)>>>, pub push:Option<std::sync::Arc<features::push::PushService>> }
impl AppState {pub fn new(db:DatabaseConnection,config:config::Config)->Self{Self{db,config,mutation_limits:Default::default(),push:None}}}
