//! The application uses SeaORM. SQLx here only creates disposable PostgreSQL databases.
use purrsonnel_api::{entities::skills,migrations,repositories::directory};
use sea_orm::{ActiveModelTrait,DbBackend,EntityTrait,QueryTrait,Set,TransactionTrait};
use sea_orm_migration::MigratorTrait;
use sqlx::PgPool;
use uuid::Uuid;

#[test]
fn directory_queries_bind_search_and_keep_scope_before_pagination(){
 let actor=Uuid::new_v4();let query=directory::select(actor,&directory::Filter{q:Some("%' OR true --".into()),department:Some(Uuid::new_v4()),sort:Some("status; DROP TABLE users".into()),..Default::default()});
 let statement=query.build(DbBackend::Postgres);
 assert!(statement.sql.contains("visible_person"));assert!(statement.sql.contains("user_departments"));assert!(statement.sql.contains("display_name"));
 assert!(!statement.sql.contains("DROP TABLE"));assert!(!statement.sql.contains("OR true --"));assert!(statement.values.is_some());
}
#[test]
fn directory_pending_terms_uses_correlated_applicability_and_signature_queries(){
 let query=directory::select(Uuid::new_v4(),&directory::Filter{pending_terms:Some(true),..Default::default()}).build(DbBackend::Postgres);
 for expected in ["term_versions","term_signatures","applicable_term","EXISTS"]{assert!(query.sql.contains(expected),"{expected}");}
}
#[test]
fn seaorm_errors_do_not_expose_query_values_to_api_clients(){
 let error=purrsonnel_api::error::ApiError::from(sea_orm::DbErr::Custom("secret personnel values".into()));
 assert_eq!(error.0,axum::http::StatusCode::INTERNAL_SERVER_ERROR);assert!(!error.1.contains("secret"));
 assert_eq!(purrsonnel_api::error::ApiError::from(sea_orm::DbErr::RecordNotFound("missing".into())).0,axum::http::StatusCode::NOT_FOUND);
}
#[sqlx::test(migrations=false)]
async fn seaorm_migrations_bootstrap_once_and_register_all_versions(db:PgPool){
 let orm=sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(db);
 migrations::up(&orm).await.unwrap();migrations::up(&orm).await.unwrap();
 assert_eq!(migrations::Migrator::get_applied_migrations(&orm).await.unwrap().len(),5);
 assert_eq!(skills::Entity::find().all(&orm).await.unwrap().len(),0);
}
#[sqlx::test(migrations=false)]
async fn active_models_preserve_database_defaults_and_transaction_rollback(db:PgPool){
 let orm=sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(db);migrations::up(&orm).await.unwrap();
 let tx=orm.begin().await.unwrap();let skill=skills::ActiveModel{name:Set("Temporary skill".into()),..Default::default()}.insert(&tx).await.unwrap();assert!(!skill.id.is_nil());
 tx.rollback().await.unwrap();assert!(skills::Entity::find_by_id(skill.id).one(&orm).await.unwrap().is_none());
}
#[sqlx::test(migrations="./migrations")]
async fn legacy_history_is_rejected_without_replaying_or_deleting_evidence(db:PgPool){
 let orm=sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(db);
 let error=migrations::up(&orm).await.unwrap_err();assert!(error.to_string().contains("Legacy SQLx"));
 assert!(skills::Entity::find().all(&orm).await.is_ok());
}
