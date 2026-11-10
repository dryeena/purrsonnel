//! requests persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RECIPIENTS_SELECT_USER_DEPARTMENTS: &str = "SELECT jsonb_build_object('id',d.id,'name',d.name,'recipient_count',(SELECT count(*) FROM user_departments m JOIN users u ON u.id=m.user_id WHERE m.department_id=d.id AND m.ended_at IS NULL AND u.status='active' AND u.id<>$1)) FROM departments d WHERE d.active AND has_permission($1,'requests.send',d.id) ORDER BY d.name";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RECIPIENTS_SELECT_USERS: &str = "SELECT jsonb_build_object('id',u.id,'display_name',u.display_name,'department',d.name) FROM users u LEFT JOIN departments d ON d.id=u.primary_department_id WHERE u.status='active' AND u.id<>$1 AND visible_person($1,u.id,'requests.send') ORDER BY u.display_name LIMIT 500";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CREATE_SELECT_USERS: &str = "SELECT u.id FROM users u JOIN user_departments m ON m.user_id=u.id WHERE m.department_id=$1 AND m.ended_at IS NULL AND u.status='active' AND u.id<>$2 ORDER BY u.id FOR SHARE OF u,m";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const LIST_SELECT_REQUEST_RECIPIENTS: &str = "SELECT (to_jsonb(r)-'payload_sha256'-'idempotency_key')||jsonb_build_object('sender_name',sender.display_name,'recipient_name',COALESCE(target.display_name,d.name),'assigned_to_name',assignee.display_name,'is_recipient',EXISTS(SELECT 1 FROM request_recipients rr WHERE rr.request_id=r.id AND rr.user_id=$1),'recipient_count',(SELECT count(*) FROM request_recipients rr WHERE rr.request_id=r.id),'push_delivery',CASE WHEN r.sender_user_id=$1 THEN (SELECT jsonb_build_object('queued',count(*) FILTER(WHERE pd.status IN ('pending','processing')),'sent',count(*) FILTER(WHERE pd.status='sent'),'failed',count(*) FILTER(WHERE pd.status IN ('failed','expired'))) FROM push_deliveries pd JOIN notifications n ON n.id=pd.notification_id WHERE n.payload_json->>'request_id'=r.id::text) ELSE NULL END) FROM service_requests r JOIN users sender ON sender.id=r.sender_user_id LEFT JOIN users target ON target.id=r.recipient_user_id LEFT JOIN departments d ON d.id=r.recipient_department_id LEFT JOIN users assignee ON assignee.id=r.assigned_to_user_id WHERE (r.sender_user_id=$1 OR EXISTS(SELECT 1 FROM request_recipients rr WHERE rr.request_id=r.id AND rr.user_id=$1)) AND ($4='all' OR ($4='sent' AND r.sender_user_id=$1) OR ($4='inbox' AND EXISTS(SELECT 1 FROM request_recipients rr WHERE rr.request_id=r.id AND rr.user_id=$1))) AND ($5='all' OR ($5='active' AND r.status IN ('open','accepted')) OR r.status=$5) AND (r.title ILIKE $6 OR r.body ILIKE $6) ORDER BY r.created_at DESC,r.id LIMIT $2 OFFSET $3";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GET_SELECT_REQUEST_RECIPIENTS: &str = "SELECT (to_jsonb(r)-'payload_sha256'-'idempotency_key')||jsonb_build_object('sender_name',u.display_name,'recipient_name',COALESCE(target.display_name,d.name),'assigned_to_name',a.display_name,'is_recipient',EXISTS(SELECT 1 FROM request_recipients rr WHERE rr.request_id=r.id AND rr.user_id=$2)) FROM service_requests r JOIN users u ON u.id=r.sender_user_id LEFT JOIN users target ON target.id=r.recipient_user_id LEFT JOIN departments d ON d.id=r.recipient_department_id LEFT JOIN users a ON a.id=r.assigned_to_user_id WHERE r.id=$1 AND (r.sender_user_id=$2 OR EXISTS(SELECT 1 FROM request_recipients rr WHERE rr.request_id=r.id AND rr.user_id=$2))";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RESPOND_SELECT_REQUEST_RECIPIENTS: &str = "SELECT EXISTS(SELECT 1 FROM request_recipients WHERE request_id=$1 AND user_id=$2)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RESPOND_UPDATE_SERVICE_REQUESTS: &str = "UPDATE service_requests SET status=$2,assigned_to_user_id=CASE WHEN $2='accepted' THEN $3 ELSE assigned_to_user_id END,updated_at=now(),completed_at=CASE WHEN $2='completed' THEN now() ELSE completed_at END WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RESPOND_SELECT_REQUEST_RECIPIENTS_2: &str = "SELECT user_id FROM request_recipients WHERE request_id=$1 AND user_id<>$2 UNION SELECT $3::uuid WHERE $3<>$2";
}

pub async fn recipients_select_user_departments<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RECIPIENTS_SELECT_USER_DEPARTMENTS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn recipients_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RECIPIENTS_SELECT_USERS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_select_departments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(departments::Entity);
    query.expr(SimpleExpr::from(Expr::col(departments::Column::Id)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(departments::Column::Id))).eq(Expr::value(p1.clone()))).add(SimpleExpr::from(Expr::col(departments::Column::Active))));
    query.lock(LockType::Share);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn create_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CREATE_SELECT_USERS, vec![p1, p2]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_select_users_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Status))).eq(Expr::value("active"))));
    query.lock(LockType::Share);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_insert_service_requests<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>, p8: impl Into<Value>, p9: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let p8: Value = p8.into();
    let p9: Value = p9.into();
    let mut query = Query::insert();
    query.into_table(service_requests::Entity).columns([service_requests::Column::SenderUserId, service_requests::Column::RecipientUserId, service_requests::Column::RecipientDepartmentId, service_requests::Column::Title, service_requests::Column::Body, service_requests::Column::Urgency, service_requests::Column::DueAt, service_requests::Column::IdempotencyKey, service_requests::Column::PayloadSha256]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into(), Expr::value(p7.clone()).into(), Expr::value(p8.clone()).into(), Expr::value(p9.clone()).into()]);
    query.on_conflict(OnConflict::columns([service_requests::Column::SenderUserId, service_requests::Column::IdempotencyKey]).do_nothing().to_owned());
    query.returning(Query::returning().column(service_requests::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn create_select_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(service_requests::Entity);
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::Id)));
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::PayloadSha256)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(service_requests::Column::SenderUserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(service_requests::Column::IdempotencyKey))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn create_insert_request_recipients<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(request_recipients::Entity).columns([request_recipients::Column::RequestId, request_recipients::Column::UserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn list_select_request_recipients<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::LIST_SELECT_REQUEST_RECIPIENTS, vec![p1, p2, p3, p4, p5, p6]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn get_select_request_recipients<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GET_SELECT_REQUEST_RECIPIENTS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn respond_select_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(service_requests::Entity);
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::SenderUserId)));
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::Status)));
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::AssignedToUserId)));
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::Urgency)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(service_requests::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn respond_select_request_recipients<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RESPOND_SELECT_REQUEST_RECIPIENTS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn respond_update_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RESPOND_UPDATE_SERVICE_REQUESTS, vec![p1, p2, p3]);
    db.execute(statement).await
}

pub async fn respond_update_request_recipients<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(request_recipients::Entity);
    query.value(request_recipients::Column::ReadAt, SimpleExpr::from(Func::coalesce([SimpleExpr::from(Expr::col(request_recipients::Column::ReadAt)), Expr::current_timestamp().into()])));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(request_recipients::Column::RequestId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(request_recipients::Column::UserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn respond_select_request_recipients_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RESPOND_SELECT_REQUEST_RECIPIENTS_2, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
