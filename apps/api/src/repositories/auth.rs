//! auth persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const FROM_REQUEST_PARTS_SELECT_SESSIONS: &str = "SELECT s.user_id FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now() AND u.status='active'";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CSRF_SELECT_SESSIONS: &str = "SELECT EXISTS(SELECT 1 FROM sessions WHERE token_hash=$1 AND csrf_token=$2 AND expires_at>now())";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const ESTABLISH_INSERT_SESSIONS: &str = "INSERT INTO sessions(token_hash,user_id,csrf_token,expires_at) VALUES($1,$2,$3,now()+interval '12 hours')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const BOOTSTRAP_SELECT_SESSIONS: &str = "SELECT s.csrf_token FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now() AND u.status='active'";
}

pub async fn from_request_parts_select_sessions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::FROM_REQUEST_PARTS_SELECT_SESSIONS, vec![p1]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn can_select_policy<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("has_permission")).args([Expr::value(p1.clone()), Expr::value(p2.clone()), Expr::value(p3.clone())])));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn person_select_policy<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("visible_person")).args([Expr::value(p1.clone()), Expr::value(p2.clone()), Expr::value(p3.clone())])));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn csrf_select_sessions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CSRF_SELECT_SESSIONS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn establish_delete_sessions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::delete();
    query.from_table(sessions::Entity).cond_where(Expr::expr(SimpleExpr::from(Expr::col(sessions::Column::TokenHash))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn establish_delete_sessions_2<C: ConnectionTrait>(db: &C) -> Result<ExecResult, DbErr> {

    let mut query = Query::delete();
    query.from_table(sessions::Entity).cond_where(Expr::expr(SimpleExpr::from(Expr::col(sessions::Column::ExpiresAt))).lt(Expr::current_timestamp()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn establish_insert_sessions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::ESTABLISH_INSERT_SESSIONS, vec![p1, p2, p3]);
    db.execute(statement).await
}

pub async fn establish_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::LastLoginAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn establish_insert_audit_events<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::insert();
    query.into_table(audit_events::Entity).columns([audit_events::Column::ActorUserId, audit_events::Column::Action, audit_events::Column::ResourceType, audit_events::Column::ResourceId]).values_panic([Expr::value(p1.clone()).into(), Expr::value("auth.session_started").into(), Expr::value("user").into(), Expr::value(p1.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn bootstrap_select_sessions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::BOOTSTRAP_SELECT_SESSIONS, vec![p1]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn switch_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone()))).add(SimpleExpr::from(Expr::col(users::Column::DevIdentity))).add(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Status))).eq(Expr::value("active"))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn me_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(people_directory::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("people_directory"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(people_directory::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn me_select_permission_grants<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(permission_grants::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("permission"), SimpleExpr::from(Expr::col(permission_grants::Column::Permission)), Expr::value("scope_type"), SimpleExpr::from(Expr::col(permission_grants::Column::ScopeType)), Expr::value("scope_id"), SimpleExpr::from(Expr::col(permission_grants::Column::ScopeId))])));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(permission_grants::Column::UserId))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn me_select_users<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("id"), SimpleExpr::from(Expr::col(users::Column::Id)), Expr::value("display_name"), SimpleExpr::from(Expr::col(users::Column::DisplayName)), Expr::value("email"), SimpleExpr::from(Expr::col(users::Column::Email))])));
    query.cond_where(Condition::all().add(SimpleExpr::from(Expr::col(users::Column::DevIdentity))).add(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Status))).eq(Expr::value("active"))));
    query.order_by(users::Column::MyfwaId, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn me_select_sessions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(sessions::Entity);
    query.expr(SimpleExpr::from(Expr::col(sessions::Column::CsrfToken)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(sessions::Column::TokenHash))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn logout_delete_sessions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::delete();
    query.from_table(sessions::Entity).cond_where(Expr::expr(SimpleExpr::from(Expr::col(sessions::Column::TokenHash))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
