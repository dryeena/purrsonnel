//! people persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GET_SELECT_MANAGER_RELATIONSHIPS: &str = "SELECT jsonb_build_object('id',u.id,'display_name',u.display_name,'department_id',r.department_id) FROM manager_relationships r JOIN users u ON u.id=r.manager_user_id WHERE r.direct_user_id=$1 AND r.ended_at IS NULL";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const UPDATE_WITH_MANAGER_RELATIONSHIPS: &str = "WITH RECURSIVE chain(id) AS (SELECT $1::uuid UNION SELECT r.direct_user_id FROM manager_relationships r JOIN chain c ON r.manager_user_id=c.id WHERE r.ended_at IS NULL) SELECT EXISTS(SELECT 1 FROM chain WHERE id=$2)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DIRECTS_SELECT_PEOPLE_DIRECTORY: &str = "SELECT to_jsonb(d) FROM people_directory d JOIN manager_relationships r ON r.direct_user_id=d.id WHERE r.manager_user_id=$1 AND r.ended_at IS NULL AND (d.id=$2 OR visible_person($2,d.id,'people.read') OR $1=$2)";
}

pub async fn get_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(people_directory::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("people_directory"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(people_directory::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn get_select_manager_relationships<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GET_SELECT_MANAGER_RELATIONSHIPS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn get_select_policy<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("visible_person")).args([Expr::value(p1.clone()), Expr::value(p2.clone()), Expr::value("people.read_sensitive")])));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn get_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("email"), SimpleExpr::from(Expr::col(users::Column::Email)), Expr::value("legal_name"), SimpleExpr::from(Expr::col(users::Column::LegalName))])));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn update_select_policy<C: ConnectionTrait>(db: &C) -> Result<ExecResult, DbErr> {

    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("pg_advisory_xact_lock")).args([Expr::value(512511i32)])));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_select_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn update_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(people_directory::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("people_directory"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(people_directory::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn update_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::DisplayName, SimpleExpr::from(Func::coalesce([Expr::value(p2.clone()), SimpleExpr::from(Expr::col(users::Column::DisplayName))])));
    query.value(users::Column::Status, SimpleExpr::from(Func::coalesce([Expr::value(p3.clone()), SimpleExpr::from(Expr::col(users::Column::Status))])));
    query.value(users::Column::VolunteerEligible, SimpleExpr::from(Func::coalesce([Expr::value(p4.clone()), SimpleExpr::from(Expr::col(users::Column::VolunteerEligible))])));
    query.value(users::Column::PersonnelLevelId, SimpleExpr::from(Func::coalesce([Expr::value(p5.clone()), SimpleExpr::from(Expr::col(users::Column::PersonnelLevelId))])));
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_update_user_departments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(user_departments::Entity);
    query.value(user_departments::Column::EndedAt, Expr::current_timestamp());
    query.value(user_departments::Column::IsPrimary, Expr::value(false));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(user_departments::Column::UserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(user_departments::Column::EndedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_insert_user_departments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(user_departments::Entity).columns([user_departments::Column::UserId, user_departments::Column::DepartmentId, user_departments::Column::IsPrimary]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_update_users_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::PrimaryDepartmentId, Expr::value(p2.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_update_manager_relationships<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(manager_relationships::Entity);
    query.value(manager_relationships::Column::EndedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(manager_relationships::Column::DirectUserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(manager_relationships::Column::EndedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_with_manager_relationships<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::UPDATE_WITH_MANAGER_RELATIONSHIPS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn update_update_manager_relationships_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(manager_relationships::Entity);
    query.value(manager_relationships::Column::EndedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(manager_relationships::Column::DirectUserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(manager_relationships::Column::EndedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_insert_manager_relationships<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(manager_relationships::Entity).columns([manager_relationships::Column::ManagerUserId, manager_relationships::Column::DirectUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn directs_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DIRECTS_SELECT_PEOPLE_DIRECTORY, vec![p1, p2]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn years_select_user_service_years<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(user_service_years::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("user_service_years"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(user_service_years::Column::UserId))).eq(Expr::value(p1.clone())));
    query.order_by(user_service_years::Column::Year, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn set_year_insert_user_service_years<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(user_service_years::Entity).columns([user_service_years::Column::UserId, user_service_years::Column::Year, user_service_years::Column::VerificationStatus, user_service_years::Column::VerifiedByUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    query.on_conflict(OnConflict::columns([user_service_years::Column::UserId, user_service_years::Column::Year]).value(user_service_years::Column::VerificationStatus, Expr::value(p3.clone())).value(user_service_years::Column::VerifiedByUserId, Expr::value(p4.clone())).to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
pub async fn set_year_select_user_departments<C: ConnectionTrait, T: TryGetable>(db:&C,p1:impl Into<Value>)->Result<T,DbErr>{
 let statement=Statement::from_sql_and_values(DbBackend::Postgres,"SELECT EXISTS(SELECT 1 FROM user_departments WHERE user_id=$1 AND ended_at IS NULL AND membership_status='active')",vec![p1.into()]);db.query_one(statement).await?.ok_or_else(||DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn set_year_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn delete_year_delete_user_service_years<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::delete();
    query.from_table(user_service_years::Entity).cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(user_service_years::Column::UserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(user_service_years::Column::Year))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn delete_year_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn skills_select_skills<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(skills::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("skills"))));
    query.order_by(skills::Column::Name, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_skill_insert_skills<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::insert();
    query.into_table(skills::Entity).columns([skills::Column::Name]).values_panic([Expr::value(p1.clone()).into()]);
    query.returning(Query::returning().column(skills::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn change_skill_insert_user_skills<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(user_skills::Entity).columns([user_skills::Column::UserId, user_skills::Column::SkillId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into()]);
    query.on_conflict(OnConflict::new().do_nothing().to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn change_skill_delete_user_skills<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::delete();
    query.from_table(user_skills::Entity).cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(user_skills::Column::UserId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(user_skills::Column::SkillId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn change_skill_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
