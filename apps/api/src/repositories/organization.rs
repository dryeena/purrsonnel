//! organization persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const TREE_SELECT_USER_DEPARTMENTS: &str = "SELECT to_jsonb(d)||jsonb_build_object('lead_name',u.display_name,'member_count',(SELECT count(*) FROM user_departments m WHERE m.department_id=d.id AND m.ended_at IS NULL)) FROM departments d LEFT JOIN users u ON u.id=d.lead_user_id WHERE d.active ORDER BY d.name";
pub(super) const EVENTS_SELECT_EVENT_EDITIONS: &str = "SELECT to_jsonb(event_editions) FROM event_editions ORDER BY active DESC,year DESC,name";
}

pub async fn events_select_event_editions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {
 let statement=Statement::from_sql_and_values(DbBackend::Postgres,postgres::EVENTS_SELECT_EVENT_EDITIONS,vec![]);
 db.query_all(statement).await?.into_iter().map(|row|row.try_get_by_index(0)).collect()
}
pub async fn events_insert_event_editions<C: ConnectionTrait, T: TryGetable>(db:&C,name:String,year:i32,starts_at:Option<chrono::DateTime<chrono::Utc>>,ends_at:Option<chrono::DateTime<chrono::Utc>>,active:bool)->Result<T,DbErr>{
 let mut query=Query::insert();query.into_table(event_editions::Entity).columns([event_editions::Column::Name,event_editions::Column::Year,event_editions::Column::StartsAt,event_editions::Column::EndsAt,event_editions::Column::Active]).values_panic([Expr::value(name).into(),Expr::value(year).into(),Expr::value(starts_at).into(),Expr::value(ends_at).into(),Expr::value(active).into()]);query.returning(Query::returning().column(event_editions::Column::Id));db.query_one(db.get_database_backend().build(&query)).await?.ok_or_else(||DbErr::RecordNotFound("Event was not created".into()))?.try_get_by_index(0)
}
pub async fn events_update_event_editions<C: ConnectionTrait>(db:&C,id:uuid::Uuid,active:bool)->Result<ExecResult,DbErr>{
 let mut query=Query::update();query.table(event_editions::Entity).value(event_editions::Column::Active,Expr::value(active));query.cond_where(Expr::col(event_editions::Column::Id).eq(id));db.execute(db.get_database_backend().build(&query)).await
}

pub async fn tree_select_user_departments<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::TREE_SELECT_USER_DEPARTMENTS, vec![]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn tree_select_divisions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(divisions::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("divisions"))));
    query.cond_where(SimpleExpr::from(Expr::col(divisions::Column::Active)));
    query.order_by(divisions::Column::Name, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn tree_select_personnel_levels<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(personnel_levels::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("personnel_levels"))));
    query.cond_where(SimpleExpr::from(Expr::col(personnel_levels::Column::Active)));
    query.order_by(personnel_levels::Column::Rank, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn tree_select_position_levels<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(position_levels::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("position_levels"))));
    query.cond_where(SimpleExpr::from(Expr::col(position_levels::Column::Active)));
    query.order_by(position_levels::Column::SortOrder, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn tree_select_event_editions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(event_editions::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("event_editions"))));
    query.cond_where(SimpleExpr::from(Expr::col(event_editions::Column::Active)));
    query.order_by(event_editions::Column::Year, Order::Desc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_department_insert_departments<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(departments::Entity).columns([departments::Column::Name, departments::Column::Slug, departments::Column::DivisionId, departments::Column::Description, departments::Column::LeadUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    query.returning(Query::returning().column(departments::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_division_insert_divisions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(divisions::Entity).columns([divisions::Column::Name, divisions::Column::Description]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into()]);
    query.returning(Query::returning().column(divisions::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn fields_select_profile_field_definitions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(profile_field_definitions::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("profile_field_definitions"))));
    query.order_by(profile_field_definitions::Column::DisplayOrder, Order::Asc);
    query.order_by(profile_field_definitions::Column::Label, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
pub async fn people_page_fields_select_profile_field_definitions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {
 let mut query=Query::select();query.from(profile_field_definitions::Entity);query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("profile_field_definitions"))));query.cond_where(Condition::all().add(Expr::col(profile_field_definitions::Column::Active).eq(true)).add(Expr::col(profile_field_definitions::Column::ShowOnPeoplePage).eq(true)));query.order_by(profile_field_definitions::Column::DisplayOrder,Order::Asc);query.order_by(profile_field_definitions::Column::Label,Order::Asc);db.query_all(db.get_database_backend().build(&query)).await?.into_iter().map(|row|row.try_get_by_index(0)).collect()
}

pub async fn create_field_insert_profile_field_definitions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>, p8: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let p8: Value = p8.into();
    let mut query = Query::insert();
    query.into_table(profile_field_definitions::Entity).columns([profile_field_definitions::Column::Key, profile_field_definitions::Column::Label, profile_field_definitions::Column::Type, profile_field_definitions::Column::ChoicesJson, profile_field_definitions::Column::Visibility, profile_field_definitions::Column::EditableBy, profile_field_definitions::Column::ValidationJson, profile_field_definitions::Column::ShowOnPeoplePage]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into(), Expr::value(p7.clone()).into(), Expr::value(p8.clone()).into()]);
    query.returning(Query::returning().column(profile_field_definitions::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}
