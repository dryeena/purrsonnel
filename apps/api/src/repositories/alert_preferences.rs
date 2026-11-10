//! alert_preferences persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GET_SELECT_NOTIFICATION_PREFERENCES: &str = "SELECT to_jsonb(p)-'user_id' FROM notification_preferences p WHERE user_id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PUT_SELECT_PG_TIMEZONE_NAMES: &str = "SELECT EXISTS(SELECT 1 FROM pg_timezone_names WHERE name=$1)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PUT_UPDATE_PUSH_DELIVERIES: &str = "UPDATE push_deliveries pd SET next_attempt_at=now() FROM notifications n WHERE n.id=pd.notification_id AND n.recipient_user_id=$1 AND pd.status='pending'";
}

pub async fn get_select_notification_preferences<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GET_SELECT_NOTIFICATION_PREFERENCES, vec![p1]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn put_select_pg_timezone_names<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PUT_SELECT_PG_TIMEZONE_NAMES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn put_insert_notification_preferences<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let mut query = Query::insert();
    query.into_table(notification_preferences::Entity).columns([notification_preferences::Column::UserId, notification_preferences::Column::RequestAlerts, notification_preferences::Column::QuietEnabled, notification_preferences::Column::Timezone, notification_preferences::Column::QuietStart, notification_preferences::Column::QuietEnd]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into()]);
    query.on_conflict(OnConflict::columns([notification_preferences::Column::UserId]).value(notification_preferences::Column::RequestAlerts, Expr::value(p2.clone())).value(notification_preferences::Column::QuietEnabled, Expr::value(p3.clone())).value(notification_preferences::Column::Timezone, Expr::value(p4.clone())).value(notification_preferences::Column::QuietStart, Expr::value(p5.clone())).value(notification_preferences::Column::QuietEnd, Expr::value(p6.clone())).value(notification_preferences::Column::UpdatedAt, Expr::current_timestamp()).to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn put_update_push_deliveries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PUT_UPDATE_PUSH_DELIVERIES, vec![p1]);
    db.execute(statement).await
}
