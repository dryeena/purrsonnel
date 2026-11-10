//! openapi persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
}

pub async fn ready_select_policy<C: ConnectionTrait>(db: &C) -> Result<ExecResult, DbErr> {

    let mut query = Query::select();
    query.expr(Expr::value(1i32));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
