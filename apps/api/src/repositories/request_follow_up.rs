//! request_follow_up persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const NOTIFY_INSERT_NOTIFICATIONS: &str = "INSERT INTO notifications(recipient_user_id,type,payload_json) SELECT u.id,$2,$3::jsonb || CASE WHEN $2='request_update' THEN jsonb_build_object('status',r.status) ELSE '{}'::jsonb END FROM users u JOIN service_requests r ON r.id=$4 WHERE u.id=$1 AND u.status='active' RETURNING id";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const NOTIFY_INSERT_PUSH_DELIVERIES: &str = "INSERT INTO push_deliveries(notification_id,subscription_id,status,next_attempt_at,last_error) SELECT $1,ps.id,CASE WHEN policy.allowed_at IS NULL THEN 'suppressed' ELSE 'pending' END,COALESCE(policy.allowed_at,now()),CASE WHEN policy.allowed_at IS NULL THEN 'preferences' WHEN policy.allowed_at>now() THEN 'quiet_hours' ELSE NULL END FROM push_subscriptions ps CROSS JOIN LATERAL (SELECT request_phone_after($2,$3,now()) AS allowed_at) policy WHERE ps.user_id=$2 AND ps.active RETURNING status";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DETAILS_SELECT_SERVICE_REQUESTS: &str = "SELECT sender_user_id FROM service_requests r WHERE id=$1 AND (sender_user_id=$2 OR EXISTS(SELECT 1 FROM request_recipients WHERE request_id=r.id AND user_id=$2))";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DETAILS_SELECT_REQUEST_FOLLOW_UPS: &str = "SELECT (to_jsonb(f)-'request_id')||jsonb_build_object('escalation_name',u.display_name) FROM request_follow_ups f LEFT JOIN users u ON u.id=f.escalation_user_id WHERE f.request_id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DETAILS_SELECT_REQUEST_ACTIVITY: &str = "SELECT (to_jsonb(a)-'request_id')||jsonb_build_object('actor_name',u.display_name) FROM request_activity a LEFT JOIN users u ON u.id=a.actor_user_id WHERE request_id=$1 ORDER BY a.created_at DESC,a.id DESC LIMIT 100";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DETAILS_SELECT_PUSH_DELIVERIES: &str = "SELECT jsonb_build_object('queued',count(*) FILTER(WHERE pd.status IN ('pending','processing')),'deferred',count(*) FILTER(WHERE pd.status='pending' AND pd.next_attempt_at>now()),'sent',count(*) FILTER(WHERE pd.status='sent'),'failed',count(*) FILTER(WHERE pd.status IN ('failed','expired')),'suppressed',count(*) FILTER(WHERE pd.status='suppressed')) FROM push_deliveries pd JOIN notifications n ON n.id=pd.notification_id WHERE n.payload_json->>'request_id'=$1::text";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SCHEDULE_SELECT_USERS: &str = "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND status='active')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REMIND_SELECT_REQUEST_RECIPIENTS: &str = "SELECT rr.user_id FROM request_recipients rr JOIN users u ON u.id=rr.user_id WHERE rr.request_id=$1 AND u.status='active' AND ($2='open' OR rr.user_id=$3) ORDER BY u.id FOR SHARE OF u";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REMIND_NOW_SELECT_SERVICE_REQUESTS: &str = "SELECT sender_user_id,status,assigned_to_user_id,urgency,COALESCE(last_reminded_at,created_at)<=now()-interval '15 minutes' AS ready FROM service_requests WHERE id=$1 FOR UPDATE";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PROCESS_DUE_SELECT_SERVICE_REQUESTS: &str = "SELECT r.id,r.sender_user_id,r.status,r.assigned_to_user_id,r.urgency,r.last_reminded_at,f.reminder_at,f.escalation_at,f.escalation_user_id,f.reminder_processed_at,f.escalation_processed_at FROM service_requests r JOIN request_follow_ups f ON f.request_id=r.id WHERE (f.reminder_at<=now() AND f.reminder_processed_at IS NULL AND (r.status NOT IN ('open','accepted') OR COALESCE(r.last_reminded_at,r.created_at)<=now()-interval '15 minutes')) OR (f.escalation_at<=now() AND f.escalation_processed_at IS NULL) ORDER BY r.id FOR UPDATE OF r,f SKIP LOCKED LIMIT 1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PROCESS_DUE_SELECT_USERS: &str = "SELECT EXISTS(SELECT 1 FROM users u JOIN users sender ON sender.id=$2 WHERE u.id=$1 AND u.status='active' AND sender.status='active' AND visible_person($2,$1,'requests.send'))";
}

pub async fn activity_insert_request_activity<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(request_activity::Entity).columns([request_activity::Column::RequestId, request_activity::Column::ActorUserId, request_activity::Column::Kind, request_activity::Column::Details]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn notify_insert_notifications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::NOTIFY_INSERT_NOTIFICATIONS, vec![p1, p2, p3, p4]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn notify_insert_push_deliveries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<QueryResult>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::NOTIFY_INSERT_PUSH_DELIVERIES, vec![p1, p2, p3]);
    db.query_all(statement).await
}

pub async fn details_select_service_requests<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DETAILS_SELECT_SERVICE_REQUESTS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn details_select_request_follow_ups<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DETAILS_SELECT_REQUEST_FOLLOW_UPS, vec![p1]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn details_select_request_activity<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DETAILS_SELECT_REQUEST_ACTIVITY, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn details_select_push_deliveries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DETAILS_SELECT_PUSH_DELIVERIES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_select_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(service_requests::Entity);
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::SenderUserId)));
    query.expr(SimpleExpr::from(Expr::col(service_requests::Column::Status)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(service_requests::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn schedule_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SCHEDULE_SELECT_USERS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_insert_request_follow_ups<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(request_follow_ups::Entity).columns([request_follow_ups::Column::RequestId, request_follow_ups::Column::ReminderAt, request_follow_ups::Column::EscalationAt, request_follow_ups::Column::EscalationUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    query.on_conflict(OnConflict::columns([request_follow_ups::Column::RequestId]).value(request_follow_ups::Column::ReminderAt, Expr::value(p2.clone())).value(request_follow_ups::Column::ReminderProcessedAt, Expr::cust("NULL")).value(request_follow_ups::Column::EscalationAt, Expr::value(p3.clone())).value(request_follow_ups::Column::EscalationUserId, Expr::value(p4.clone())).value(request_follow_ups::Column::EscalationProcessedAt, Expr::cust("NULL")).value(request_follow_ups::Column::UpdatedAt, Expr::current_timestamp()).to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn remind_select_request_recipients<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REMIND_SELECT_REQUEST_RECIPIENTS, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn remind_update_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(service_requests::Entity);
    query.value(service_requests::Column::LastRemindedAt, Expr::current_timestamp());
    query.value(service_requests::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(service_requests::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn remind_now_select_service_requests<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REMIND_NOW_SELECT_SERVICE_REQUESTS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn process_due_select_service_requests<C: ConnectionTrait>(db: &C) -> Result<Option<QueryResult>, DbErr> {

    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PROCESS_DUE_SELECT_SERVICE_REQUESTS, vec![]);
    db.query_one(statement).await
}

pub async fn process_due_update_request_follow_ups<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(request_follow_ups::Entity);
    query.value(request_follow_ups::Column::ReminderProcessedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(request_follow_ups::Column::RequestId))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn process_due_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PROCESS_DUE_SELECT_USERS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn process_due_insert_request_recipients<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(request_recipients::Entity).columns([request_recipients::Column::RequestId, request_recipients::Column::UserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into()]);
    query.on_conflict(OnConflict::new().do_nothing().to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn process_due_update_request_follow_ups_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(request_follow_ups::Entity);
    query.value(request_follow_ups::Column::EscalationProcessedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(request_follow_ups::Column::RequestId))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
