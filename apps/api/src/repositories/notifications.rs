//! notifications persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;

pub async fn list_select_notifications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::select();
    query.from(notifications::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("notifications"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(notifications::Column::RecipientUserId))).eq(Expr::value(p1.clone())));
    query.order_by(notifications::Column::CreatedAt, Order::Desc);
    query.order_by(notifications::Column::Id, Order::Asc);
    query.limit(<i64 as sea_orm::sea_query::ValueType>::try_from(p2.clone()).map_err(|_| DbErr::Type("Invalid page bound".into()))? as u64);
    query.offset(<i64 as sea_orm::sea_query::ValueType>::try_from(p3.clone()).map_err(|_| DbErr::Type("Invalid page bound".into()))? as u64);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn read_update_notifications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(notifications::Entity);
    query.value(notifications::Column::ReadAt, SimpleExpr::from(Func::coalesce([SimpleExpr::from(Expr::col(notifications::Column::ReadAt)), Expr::current_timestamp().into()])));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(notifications::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(notifications::Column::RecipientUserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
