//! dashboard persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DASHBOARD_SELECT_SERVICE_REQUESTS: &str = "SELECT jsonb_build_object('incoming_requests',(SELECT count(*) FROM service_requests sr JOIN request_recipients rr ON rr.request_id=sr.id WHERE rr.user_id=$1 AND sr.status IN ('open','accepted')),'people',(SELECT count(*) FROM users u WHERE visible_person($1,u.id,'people.read')),'staff',(SELECT count(*) FROM users u WHERE status='active' AND personnel_level_id IS NOT NULL AND visible_person($1,u.id,'people.read')),'pending_approvals',(SELECT count(*) FROM volunteer_entries e WHERE e.status='submitted' AND has_permission($1,'volunteer.time.approve',e.department_id)),'applications',(SELECT count(*) FROM applications a JOIN position_postings pos ON pos.id=a.position_posting_id WHERE a.status IN ('submitted','screening','interviewing') AND (a.applicant_user_id=$1 OR has_permission($1,'applications.review',pos.department_id))),'volunteer_minutes',(SELECT COALESCE(sum(minutes_worked),0) FROM volunteer_entries WHERE volunteer_user_id=$1 AND status='approved'),'pending_terms',(SELECT count(*) FROM term_versions v WHERE v.published_at IS NOT NULL AND v.superseded_at IS NULL AND applicable_term(v.term_id,$1) AND NOT EXISTS(SELECT 1 FROM term_signatures sig WHERE sig.term_version_id=v.id AND sig.user_id=$1)))";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DASHBOARD_SELECT_DEPARTMENTS: &str = "SELECT jsonb_build_object('name',d.name,'count',count(m.id)) FROM departments d LEFT JOIN user_departments m ON m.department_id=d.id AND m.ended_at IS NULL WHERE has_permission($1,'people.read',d.id) GROUP BY d.id ORDER BY count(m.id) DESC";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const AUDIT_SELECT_AUDIT_EVENTS: &str = "SELECT to_jsonb(e)||jsonb_build_object('actor_name',u.display_name) FROM audit_events e LEFT JOIN users u ON u.id=e.actor_user_id ORDER BY e.created_at DESC,e.id DESC LIMIT $1 OFFSET $2";
}

pub async fn dashboard_select_service_requests<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DASHBOARD_SELECT_SERVICE_REQUESTS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn dashboard_select_departments<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DASHBOARD_SELECT_DEPARTMENTS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn dashboard_select_notifications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(notifications::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("id"), SimpleExpr::from(Expr::col(notifications::Column::Id)), Expr::value("type"), SimpleExpr::from(Expr::col(notifications::Column::Type)), Expr::value("payload"), SimpleExpr::from(Expr::col(notifications::Column::PayloadJson)), Expr::value("created_at"), SimpleExpr::from(Expr::col(notifications::Column::CreatedAt))])));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(notifications::Column::RecipientUserId))).eq(Expr::value(p1.clone())));
    query.order_by(notifications::Column::CreatedAt, Order::Desc);
    query.limit(5);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn audit_select_audit_events<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::AUDIT_SELECT_AUDIT_EVENTS, vec![p1, p2]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
