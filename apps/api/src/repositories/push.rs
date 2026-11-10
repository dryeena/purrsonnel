//! push persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SUBSCRIBE_INSERT_PUSH_SUBSCRIPTIONS: &str = "INSERT INTO push_subscriptions(user_id,endpoint_hash,endpoint,p256dh,auth) VALUES($1,$2,$3,$4,$5) ON CONFLICT(endpoint_hash) DO UPDATE SET p256dh=$4,auth=$5,active=true,updated_at=now() WHERE push_subscriptions.user_id=$1 OR NOT push_subscriptions.active RETURNING id";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SUBSCRIBE_UPDATE_PUSH_DELIVERIES: &str = "UPDATE push_deliveries pd SET status='expired',last_error='device_changed' FROM notifications n WHERE pd.subscription_id=$1 AND n.id=pd.notification_id AND n.recipient_user_id<>$2 AND pd.status IN ('pending','processing')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SUBSCRIPTION_STATUS_SELECT_PUSH_SUBSCRIPTIONS: &str = "SELECT EXISTS(SELECT 1 FROM push_subscriptions WHERE endpoint_hash=$1 AND user_id=$2 AND active)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const UNSUBSCRIBE_UPDATE_PUSH_DELIVERIES: &str = "UPDATE push_deliveries pd SET status='expired',last_error='device_disabled' FROM push_subscriptions ps WHERE ps.id=pd.subscription_id AND ps.endpoint_hash=$1 AND ps.user_id=$2 AND pd.status IN ('pending','processing')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DELIVER_ONE_SELECT_PUSH_DELIVERIES: &str = "SELECT id FROM push_deliveries WHERE (status='pending' AND next_attempt_at<=now()) OR (status='processing' AND leased_until<now()) ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DELIVER_ONE_SELECT_PUSH_DELIVERIES_2: &str = "SELECT ps.endpoint,ps.p256dh,ps.auth,ps.id AS subscription_id,ps.active,ps.user_id=n.recipient_user_id AS same_owner,u.status='active' AS active_user,pd.attempts,pd.expires_at,n.id AS notification_id,n.type AS notification_type,n.payload_json,request_phone_after(n.recipient_user_id,COALESCE(r.urgency,'normal'),now()) AS allowed_after,CASE WHEN n.type IN ('service_request','request_escalated') THEN r.status<>'open' WHEN n.type='request_reminder' THEN r.status NOT IN ('open','accepted') ELSE false END AS obsolete FROM push_deliveries pd JOIN push_subscriptions ps ON ps.id=pd.subscription_id JOIN notifications n ON n.id=pd.notification_id JOIN users u ON u.id=n.recipient_user_id LEFT JOIN service_requests r ON r.id::text=n.payload_json->>'request_id' WHERE pd.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DELIVER_ONE_UPDATE_PUSH_DELIVERIES_3: &str = "UPDATE push_deliveries SET status='processing',leased_until=now()+interval '60 seconds',lease_token=$2,attempts=attempts+1 WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DELIVER_ONE_UPDATE_PUSH_DELIVERIES_4: &str = "UPDATE push_deliveries SET status=$2,last_error=$3,sent_at=CASE WHEN $2='sent' THEN now() ELSE NULL END,leased_until=NULL,lease_token=NULL,next_attempt_at=now()+make_interval(secs=>LEAST(900,power(2,LEAST(attempts,5))::int*15)) WHERE id=$1 AND status='processing' AND lease_token=$4";
}

pub async fn subscribe_insert_push_subscriptions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SUBSCRIBE_INSERT_PUSH_SUBSCRIPTIONS, vec![p1, p2, p3, p4, p5]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn subscribe_update_push_subscriptions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(push_subscriptions::Entity);
    query.value(push_subscriptions::Column::UserId, Expr::value(p2.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(push_subscriptions::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn subscribe_update_push_deliveries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SUBSCRIBE_UPDATE_PUSH_DELIVERIES, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn subscription_status_select_push_subscriptions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SUBSCRIPTION_STATUS_SELECT_PUSH_SUBSCRIPTIONS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn unsubscribe_update_push_subscriptions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(push_subscriptions::Entity);
    query.value(push_subscriptions::Column::Active, Expr::value(false));
    query.value(push_subscriptions::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(push_subscriptions::Column::EndpointHash))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(push_subscriptions::Column::UserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn unsubscribe_update_push_deliveries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::UNSUBSCRIBE_UPDATE_PUSH_DELIVERIES, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn deliver_one_select_push_deliveries<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Option<T>, DbErr> {

    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DELIVER_ONE_SELECT_PUSH_DELIVERIES, vec![]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn deliver_one_select_push_deliveries_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DELIVER_ONE_SELECT_PUSH_DELIVERIES_2, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn deliver_one_update_push_deliveries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::update();
    query.table(push_deliveries::Entity);
    query.value(push_deliveries::Column::Status, Expr::value(p2.clone()));
    query.value(push_deliveries::Column::LastError, Expr::value(p3.clone()));
    query.value(push_deliveries::Column::LeasedUntil, Expr::cust("NULL"));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(push_deliveries::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn deliver_one_update_push_deliveries_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(push_deliveries::Entity);
    query.value(push_deliveries::Column::Status, Expr::value("pending"));
    query.value(push_deliveries::Column::NextAttemptAt, Expr::value(p2.clone()));
    query.value(push_deliveries::Column::LeasedUntil, Expr::cust("NULL"));
    query.value(push_deliveries::Column::LastError, Expr::value("quiet_hours"));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(push_deliveries::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn deliver_one_update_push_deliveries_3<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DELIVER_ONE_UPDATE_PUSH_DELIVERIES_3, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn deliver_one_update_push_deliveries_4<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DELIVER_ONE_UPDATE_PUSH_DELIVERIES_4, vec![p1, p2, p3, p4]);
    db.execute(statement).await
}

pub async fn deliver_one_update_push_subscriptions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(push_subscriptions::Entity);
    query.value(push_subscriptions::Column::Active, Expr::value(false));
    query.value(push_subscriptions::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(push_subscriptions::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
