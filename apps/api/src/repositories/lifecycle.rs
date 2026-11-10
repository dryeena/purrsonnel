//! lifecycle persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PENDING_PERSON_SELECT_POSITION_POSTINGS: &str = "SELECT pos.department_id,f.schema_json FROM position_postings pos JOIN form_template_versions f ON f.id=pos.form_template_version_id WHERE pos.id=$1 AND pos.status='open' FOR SHARE OF pos";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DEPARTMENT_UPDATE_DEPARTMENTS: &str = "UPDATE departments SET name=COALESCE($2,name),description=COALESCE($3,description),lead_user_id=CASE WHEN $5 THEN NULL ELSE COALESCE($4,lead_user_id) END,active=COALESCE($6,active) WHERE id=$1";
pub(super) const DEPARTMENT_SELECT_USER_DEPARTMENTS: &str = "SELECT EXISTS(SELECT 1 FROM user_departments WHERE department_id=$1 AND user_id=$2 AND ended_at IS NULL)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DIVISION_UPDATE_DIVISIONS: &str = "UPDATE divisions SET name=COALESCE($2,name),description=COALESCE($3,description),lead_user_id=CASE WHEN $5 THEN NULL ELSE COALESCE($4,lead_user_id) END,active=COALESCE($6,active) WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const INTERVIEW_SELECT_INTERVIEWS: &str = "SELECT pos.department_id FROM interviews i JOIN applications a ON a.id=i.application_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE i.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const INTERVIEW_SELECT_PG_TIMEZONE_NAMES: &str = "SELECT EXISTS(SELECT 1 FROM pg_timezone_names WHERE name=$1)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const ADD_INTERVIEWER_SELECT_INTERVIEWS: &str = "SELECT pos.department_id FROM interviews i JOIN applications a ON a.id=i.application_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE i.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REOPEN_SELECT_VOLUNTEER_ENTRIES: &str = "SELECT department_id,status,to_jsonb(e) AS snapshot FROM volunteer_entries e WHERE id=$1 FOR UPDATE";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const EDIT_ENTRY_SELECT_VOLUNTEER_ENTRIES_2: &str = "SELECT EXISTS(SELECT 1 FROM volunteer_entries WHERE volunteer_user_id=$1 AND id<>$4 AND status IN ('draft','submitted','approved') AND started_at<$3 AND ended_at>$2)";
}

pub async fn pending_person_insert_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(users::Entity).columns([users::Column::DisplayName, users::Column::Email, users::Column::Status]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value("pending_identity").into()]);
    query.returning(Query::returning().column(users::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn pending_person_select_position_postings<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PENDING_PERSON_SELECT_POSITION_POSTINGS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn pending_person_insert_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(applications::Entity).columns([applications::Column::PositionPostingId, applications::Column::ApplicantUserId, applications::Column::SubmittedMyfwaId, applications::Column::AnswersJson, applications::Column::Status, applications::Column::SubmittedAt]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value("screening").into(), Expr::current_timestamp().into()]);
    query.returning(Query::returning().column(applications::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn department_update_departments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DEPARTMENT_UPDATE_DEPARTMENTS, vec![p1, p2, p3, p4, p5, p6]);
    db.execute(statement).await
}

pub async fn department_select_user_departments<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DEPARTMENT_SELECT_USER_DEPARTMENTS, vec![p1.into(), p2.into()]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn division_update_divisions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DIVISION_UPDATE_DIVISIONS, vec![p1, p2, p3, p4, p5, p6]);
    db.execute(statement).await
}

pub async fn update_list_update_saved_lists<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::update();
    query.table(saved_lists::Entity);
    query.value(saved_lists::Column::Name, Expr::value(p3.clone()));
    query.value(saved_lists::Column::FilterAstJson, Expr::value(p4.clone()));
    query.value(saved_lists::Column::SharedScope, Expr::value(p5.clone()));
    query.value(saved_lists::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(saved_lists::Column::OwnerUserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn interview_select_interviews<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::INTERVIEW_SELECT_INTERVIEWS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn interview_select_pg_timezone_names<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::INTERVIEW_SELECT_PG_TIMEZONE_NAMES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn interview_update_interviews<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let mut query = Query::update();
    query.table(interviews::Entity);
    query.value(interviews::Column::ScheduledStartAt, Expr::value(p2.clone()));
    query.value(interviews::Column::ScheduledEndAt, Expr::value(p3.clone()));
    query.value(interviews::Column::Timezone, Expr::value(p4.clone()));
    query.value(interviews::Column::LocationLabel, Expr::value(p5.clone()));
    query.value(interviews::Column::LocationUrl, Expr::value(p6.clone()));
    query.value(interviews::Column::Status, Expr::value(p7.clone()));
    query.value(interviews::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(interviews::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn add_interviewer_select_interviews<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::ADD_INTERVIEWER_SELECT_INTERVIEWS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn add_interviewer_insert_interview_participants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(interview_participants::Entity).columns([interview_participants::Column::InterviewId, interview_participants::Column::InterviewerUserId, interview_participants::Column::AssignedByUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.on_conflict(OnConflict::new().do_nothing().to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn owner_select_position_postings<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(position_postings::Entity);
    query.expr(SimpleExpr::from(Expr::col(position_postings::Column::DepartmentId)));
    query.expr(SimpleExpr::from(Expr::col(position_postings::Column::HiringOwnerUserId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn owner_update_position_postings<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(position_postings::Entity);
    query.value(position_postings::Column::HiringOwnerUserId, Expr::value(p2.clone()));
    query.value(position_postings::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn reopen_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REOPEN_SELECT_VOLUNTEER_ENTRIES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn reopen_insert_volunteer_adjustments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(volunteer_adjustments::Entity).columns([volunteer_adjustments::Column::EntryId, volunteer_adjustments::Column::ActorUserId, volunteer_adjustments::Column::Reason, volunteer_adjustments::Column::PreviousJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn reopen_update_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(volunteer_entries::Entity);
    query.value(volunteer_entries::Column::Status, Expr::value("draft"));
    query.value(volunteer_entries::Column::SubmittedAt, Expr::cust("NULL"));
    query.value(volunteer_entries::Column::DecidedAt, Expr::cust("NULL"));
    query.value(volunteer_entries::Column::DecidedByUserId, Expr::cust("NULL"));
    query.value(volunteer_entries::Column::DecisionNote, Expr::cust("NULL"));
    query.value(volunteer_entries::Column::UpdatedAt, Expr::current_timestamp());
    query.value(volunteer_entries::Column::Version, Expr::col(volunteer_entries::Column::Version).add(1i32));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn reopen_update_volunteer_approval_tokens<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(volunteer_approval_tokens::Entity);
    query.value(volunteer_approval_tokens::Column::RevokedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::EntryId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RevokedAt))).is_null()).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RedeemedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn edit_entry_select_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn edit_entry_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_entries::Entity);
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::VolunteerUserId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Status)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn edit_entry_select_volunteer_entries_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::EDIT_ENTRY_SELECT_VOLUNTEER_ENTRIES_2, vec![p1, p2, p3, p4]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn edit_entry_update_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>, p8: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let p8: Value = p8.into();
    let mut query = Query::update();
    query.table(volunteer_entries::Entity);
    query.value(volunteer_entries::Column::DepartmentId, Expr::value(p2.clone()));
    query.value(volunteer_entries::Column::EventEditionId, Expr::value(p3.clone()));
    query.value(volunteer_entries::Column::WorkDate, Expr::value(p4.clone()));
    query.value(volunteer_entries::Column::MinutesWorked, Expr::value(p5.clone()));
    query.value(volunteer_entries::Column::StartedAt, Expr::value(p6.clone()));
    query.value(volunteer_entries::Column::EndedAt, Expr::value(p7.clone()));
    query.value(volunteer_entries::Column::WorkDescription, Expr::value(p8.clone()));
    query.value(volunteer_entries::Column::UpdatedAt, Expr::current_timestamp());
    query.value(volunteer_entries::Column::Version, Expr::col(volunteer_entries::Column::Version).add(1i32));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn job_select_bulk_change_jobs<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(bulk_change_jobs::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("bulk_change_jobs"))));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(bulk_change_jobs::Column::CreatorUserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}
