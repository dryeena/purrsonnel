//! detail persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const ENTRY_SELECT_VOLUNTEER_ENTRIES: &str = "SELECT volunteer_user_id,department_id,to_jsonb(e) AS data FROM volunteer_entries e WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const POSITION_SELECT_POSITION_POSTINGS: &str = "SELECT pos.department_id,pos.status,to_jsonb(pos)||jsonb_build_object('department',d.name,'level',l.label,'questions',f.schema_json) AS data FROM position_postings pos JOIN departments d ON d.id=pos.department_id JOIN position_levels l ON l.id=pos.position_level_id JOIN form_template_versions f ON f.id=pos.form_template_version_id WHERE pos.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const INTERVIEW_SELECT_INTERVIEW_PARTICIPANTS: &str = "SELECT pos.department_id,a.applicant_user_id,EXISTS(SELECT 1 FROM interview_participants ip WHERE ip.interview_id=i.id AND ip.interviewer_user_id=$2) AS panel_member,to_jsonb(i)||jsonb_build_object('applicant_name',u.display_name,'position',pos.title,'panel',COALESCE((SELECT jsonb_agg(jsonb_build_object('user_id',ip.interviewer_user_id,'response',ip.response)) FROM interview_participants ip WHERE ip.interview_id=i.id),'[]')) AS data FROM interviews i JOIN applications a ON a.id=i.application_id JOIN users u ON u.id=a.applicant_user_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE i.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const EXPORT_PERSON_SELECT_USERS: &str = "SELECT to_jsonb(u)-'dev_identity' FROM users u WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const EXPORT_PERSON_SELECT_TERM_SIGNATURES: &str = "SELECT to_jsonb(sig)||jsonb_build_object('title',t.title,'body_markdown',v.body_markdown,'version_number',v.version_number) FROM term_signatures sig JOIN term_versions v ON v.id=sig.term_version_id JOIN terms t ON t.id=v.term_id WHERE sig.user_id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const EXPORT_PERSON_SELECT_PROFILE_FIELD_VALUES: &str = "SELECT jsonb_build_object('key',f.key,'label',f.label,'value',v.value_json,'updated_at',v.updated_at) FROM profile_field_values v JOIN profile_field_definitions f ON f.id=v.definition_id WHERE v.user_id=$1";
}

pub async fn entry_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::ENTRY_SELECT_VOLUNTEER_ENTRIES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn position_select_position_postings<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::POSITION_SELECT_POSITION_POSTINGS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn interview_select_interview_participants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::INTERVIEW_SELECT_INTERVIEW_PARTICIPANTS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn saved_list_select_saved_lists<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(saved_lists::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("saved_lists"))));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::Id))).eq(Expr::value(p1.clone()))).add(Condition::any().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::OwnerUserId))).eq(Expr::value(p2.clone()))).add(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::SharedScope))).eq(Expr::value("hr"))).add(SimpleExpr::from(Func::cust(Alias::new("has_permission")).args([Expr::value(p2.clone()), Expr::value("people.read"), Expr::cust("NULL")]))))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn export_person_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::EXPORT_PERSON_SELECT_USERS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn export_person_select_term_signatures<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::EXPORT_PERSON_SELECT_TERM_SIGNATURES, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn export_person_select_volunteer_entries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_entries::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("volunteer_entries"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::VolunteerUserId))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn export_person_select_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(applications::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("applications"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::ApplicantUserId))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn export_person_select_people_directory<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(people_directory::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("people_directory"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(people_directory::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn export_person_select_user_service_years<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(user_service_years::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("user_service_years"))));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(user_service_years::Column::UserId))).eq(Expr::value(p1.clone())));
    query.order_by(user_service_years::Column::Year, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn export_person_select_profile_field_values<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::EXPORT_PERSON_SELECT_PROFILE_FIELD_VALUES, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
