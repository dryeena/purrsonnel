//! SeaORM owns migration history. PostgreSQL policy/trigger DDL stays in reviewed
//! SQL files; these are immutable, forward-only migration inputs, not route queries.
use sea_orm::{ConnectionTrait,DatabaseConnection,DbBackend,DbErr,Statement,TransactionTrait};
use sea_orm_migration::{MigrationName,MigrationTrait,MigratorTrait,SchemaManager};

pub struct Migrator;
struct SqlMigration {name:&'static str,sql:&'static str}
impl MigrationName for SqlMigration {fn name(&self)->&str{self.name}}
#[async_trait::async_trait]
impl MigrationTrait for SqlMigration {
 async fn up(&self,manager:&SchemaManager)->Result<(),DbErr>{manager.get_connection().execute_unprepared(self.sql).await?;Ok(())}
 async fn down(&self,_:&SchemaManager)->Result<(),DbErr>{Err(DbErr::Migration("Personnel evidence migrations are forward-only. Restore a reviewed backup instead of deleting immutable evidence.".into()))}
}
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
 fn migrations()->Vec<Box<dyn MigrationTrait>>{
  vec![
   Box::new(SqlMigration{name:"m20261109_000001_schema",sql:include_str!("../migrations/0001_schema.sql")}),
   Box::new(SqlMigration{name:"m20261109_000002_seed_function",sql:include_str!("../migrations/0002_seed_function.sql")}),
   Box::new(SqlMigration{name:"m20261109_000003_adjustments",sql:include_str!("../migrations/0003_adjustments.sql")}),
   Box::new(SqlMigration{name:"m20261109_000004_requests_push",sql:include_str!("../migrations/0004_requests_push.sql")}),
   Box::new(SqlMigration{name:"m20261109_000005_follow_through",sql:include_str!("../migrations/0005_request_follow_through.sql")}),
  ]
 }
}
pub async fn up(db:&DatabaseConnection)->Result<(),DbErr>{
 let tx=db.begin().await?;
 tx.execute(Statement::from_string(DbBackend::Postgres,"SELECT pg_advisory_xact_lock(512509)".to_owned())).await?;
 let legacy=tx.query_one(Statement::from_string(DbBackend::Postgres,"SELECT to_regclass('public._sqlx_migrations') IS NOT NULL AS legacy".to_owned())).await?.ok_or_else(||DbErr::Custom("Unable to inspect migration history".into()))?.try_get::<bool>("","legacy")?;
 if legacy{return Err(DbErr::Migration("Legacy SQLx migration history detected. Do not replay the schema. Follow docs/SEAORM.md for the reviewed database-transfer procedure.".into()));}
 Migrator::up(&tx,None).await?;
 tx.commit().await
}
