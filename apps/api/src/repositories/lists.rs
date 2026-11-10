//! lists persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const EXECUTE_SELECT_USERS: &str = "SELECT to_jsonb(u)->'updated_at' FROM users u WHERE id=$1 FOR UPDATE";
}

pub async fn targets_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(people_directory::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("people_directory"))));
    query.cond_where(SimpleExpr::from(Func::cust(Alias::new("visible_person")).args([Expr::value(p1.clone()), SimpleExpr::from(Expr::col(people_directory::Column::Id)), Expr::value(p2.clone())])));
    query.order_by(people_directory::Column::DisplayName, Order::Asc);
    query.order_by(people_directory::Column::Id, Order::Asc);
    query.limit(1001);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn list_select_saved_lists<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(saved_lists::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("saved_lists"))));
    query.cond_where(Condition::any().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::OwnerUserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::SharedScope))).eq(Expr::value("hr"))));
    query.order_by(saved_lists::Column::Name, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_insert_saved_lists<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(saved_lists::Entity).columns([saved_lists::Column::Name, saved_lists::Column::OwnerUserId, saved_lists::Column::FilterAstJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(saved_lists::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn run_select_saved_lists<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(saved_lists::Entity);
    query.expr(SimpleExpr::from(Expr::col(saved_lists::Column::FilterAstJson)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::Id))).eq(Expr::value(p1.clone()))).add(Condition::any().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::OwnerUserId))).eq(Expr::value(p2.clone()))).add(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::SharedScope))).eq(Expr::value("hr"))).add(SimpleExpr::from(Func::cust(Alias::new("has_permission")).args([Expr::value(p2.clone()), Expr::value("people.read"), Expr::cust("NULL")]))))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn delete_delete_saved_lists<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::delete();
    query.from_table(saved_lists::Entity).cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::OwnerUserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn preview_insert_bulk_change_jobs<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(bulk_change_jobs::Entity).columns([bulk_change_jobs::Column::CreatorUserId, bulk_change_jobs::Column::RequestedFilterSnapshotJson, bulk_change_jobs::Column::TargetUserIdsJson, bulk_change_jobs::Column::OperationJson, bulk_change_jobs::Column::PreviewCount]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    query.returning(Query::returning().column(bulk_change_jobs::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn execute_select_bulk_change_jobs<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(bulk_change_jobs::Entity);
    query.expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::TargetUserIdsJson)));
    query.expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::OperationJson)));
    query.expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::Status)));
    query.expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::CreatedAt)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::CreatorUserId))).eq(Expr::value(p2.clone()))));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn execute_select_policy<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("visible_person")).args([Expr::value(p1.clone()), Expr::value(p2.clone()), Expr::value("people.edit")])));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn execute_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::EXECUTE_SELECT_USERS, vec![p1]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn execute_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::Status, Expr::value(p2.clone()));
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn execute_update_users_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::VolunteerEligible, Expr::value(p2.clone()));
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn execute_update_bulk_change_jobs<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::update();
    query.table(bulk_change_jobs::Entity);
    query.value(bulk_change_jobs::Column::Status, Expr::value(p2.clone()));
    query.value(bulk_change_jobs::Column::CompletedAt, Expr::current_timestamp());
    query.value(bulk_change_jobs::Column::ResultJson, Expr::value(p3.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn jobs_select_bulk_change_jobs<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(bulk_change_jobs::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("bulk_change_jobs"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::CreatorUserId))).eq(Expr::value(p1.clone())));
    query.order_by(bulk_change_jobs::Column::CreatedAt, Order::Desc);
    query.limit(50);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
