use purrsonnel_api::{app,config::Config,migrations,repositories,AppState};
use sea_orm::{ConnectOptions,Database,TransactionTrait};
use std::time::Duration;

#[tokio::main]
async fn main()->Result<(),Box<dyn std::error::Error>>{
 dotenvy::dotenv().ok();
 tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
 let config=Config::from_env()?;
 let push=purrsonnel_api::features::push::PushService::from_env()?;
 let mut options=ConnectOptions::new(std::env::var("DATABASE_URL")?);
 options.max_connections(10).min_connections(1).connect_timeout(Duration::from_secs(10)).acquire_timeout(Duration::from_secs(10)).sqlx_logging(false);
 let db=Database::connect(options).await?;
 migrations::up(&db).await?;
 if config.dev_auth{
  let tx=db.begin().await?;
  repositories::startup::main_select_policy(&tx).await?;
  repositories::startup::main_select_policy_2(&tx).await?;
  tx.commit().await?;
 }
 let listener=tokio::net::TcpListener::bind(&config.bind).await?;
 tracing::info!(bind=%config.bind,version=env!("CARGO_PKG_VERSION"),"Purrsonnel API ready");
 let mut state=AppState::new(db,config);state.push=push;
 let push_worker=tokio::spawn(purrsonnel_api::features::push::run(state.clone()));
 let follow_ups=tokio::spawn(purrsonnel_api::features::request_follow_up::run(state.clone()));
 axum::serve(listener,app::router(state.clone())).with_graceful_shutdown(shutdown()).await?;
 push_worker.abort();follow_ups.abort();
 let _=tokio::join!(push_worker,follow_ups);
 state.db.close().await?;
 Ok(())
}
async fn shutdown(){
 #[cfg(unix)]{
  let mut terminate=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("Unable to register termination handler");
  tokio::select!{_=tokio::signal::ctrl_c()=>{},_=terminate.recv()=>{}}
 }
 #[cfg(not(unix))]{let _=tokio::signal::ctrl_c().await;}
}
