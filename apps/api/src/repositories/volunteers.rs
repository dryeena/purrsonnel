//! volunteers persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const LIST_SELECT_VOLUNTEER_ENTRIES: &str = "SELECT to_jsonb(e)||jsonb_build_object('volunteer_name',u.display_name,'department',d.name,'event',ev.name,'decided_by_name',a.display_name) FROM volunteer_entries e JOIN users u ON u.id=e.volunteer_user_id JOIN departments d ON d.id=e.department_id LEFT JOIN event_editions ev ON ev.id=e.event_edition_id LEFT JOIN users a ON a.id=e.decided_by_user_id WHERE e.volunteer_user_id=$1 OR has_permission($1,'volunteer.time.approve',e.department_id) OR has_permission($1,'volunteer.time.admin',e.department_id) ORDER BY e.created_at DESC,e.id LIMIT $2 OFFSET $3";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CREATE_SELECT_USERS: &str = "SELECT volunteer_eligible AND myfwa_id IS NOT NULL AND status='active' FROM users WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CREATE_SELECT_VOLUNTEER_ENTRIES: &str = "SELECT EXISTS(SELECT 1 FROM volunteer_entries WHERE volunteer_user_id=$1 AND status IN ('draft','submitted','approved') AND started_at<$3 AND ended_at>$2)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const TRANSITION_SELECT_USERS: &str = "SELECT volunteer_eligible AND myfwa_id IS NOT NULL AND status='active' FROM users WHERE id=$1 FOR SHARE";
pub(super) const APPROVER_SELECT_USERS: &str = "SELECT status='active' AND personnel_level_id IS NOT NULL FROM users WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const TRANSITION_UPDATE_VOLUNTEER_ENTRIES: &str = "UPDATE volunteer_entries SET status=$2,submitted_at=CASE WHEN $2='submitted' THEN now() ELSE submitted_at END,version=version+1,updated_at=now() WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const QR_INSERT_VOLUNTEER_APPROVAL_TOKENS: &str = "INSERT INTO volunteer_approval_tokens(entry_id,token_hash,expires_at) VALUES($1,$2,now()+interval '10 minutes') RETURNING expires_at";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PREVIEW_SELECT_VOLUNTEER_APPROVAL_TOKENS: &str = "SELECT e.department_id,e.volunteer_user_id,to_jsonb(e)||jsonb_build_object('volunteer_name',u.display_name,'department',d.name) AS data FROM volunteer_approval_tokens t JOIN volunteer_entries e ON e.id=t.entry_id JOIN users u ON u.id=e.volunteer_user_id JOIN departments d ON d.id=e.department_id WHERE t.token_hash=$1 AND t.expires_at>now() AND t.revoked_at IS NULL AND t.redeemed_at IS NULL AND e.status='submitted'";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const DECIDE_SELECT_VOLUNTEER_APPROVAL_TOKENS_2: &str = "SELECT EXISTS(SELECT 1 FROM volunteer_approval_tokens WHERE token_hash=$1 AND expires_at>now() AND revoked_at IS NULL AND redeemed_at IS NULL)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REPORT_SELECT_VOLUNTEER_ENTRIES: &str = "SELECT jsonb_build_object('user_id',u.id,'display_name',u.display_name,'department',d.name,'event_edition_id',ev.id,'event',ev.name,'minutes',sum(e.minutes_worked)) FROM volunteer_entries e JOIN users u ON u.id=e.volunteer_user_id JOIN departments d ON d.id=e.department_id LEFT JOIN event_editions ev ON ev.id=e.event_edition_id WHERE e.status='approved' AND (e.volunteer_user_id=$1 OR has_permission($1,'volunteer.time.approve',e.department_id) OR has_permission($1,'volunteer.time.admin',e.department_id)) GROUP BY u.id,d.id,ev.id ORDER BY d.name,u.display_name";
}

pub async fn list_select_volunteer_entries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::LIST_SELECT_VOLUNTEER_ENTRIES, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CREATE_SELECT_USERS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_select_users_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn create_select_volunteer_entries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CREATE_SELECT_VOLUNTEER_ENTRIES, vec![p1, p2, p3]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_insert_volunteer_entries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>, p8: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let p8: Value = p8.into();
    let mut query = Query::insert();
    query.into_table(volunteer_entries::Entity).columns([volunteer_entries::Column::VolunteerUserId, volunteer_entries::Column::EventEditionId, volunteer_entries::Column::DepartmentId, volunteer_entries::Column::WorkDate, volunteer_entries::Column::MinutesWorked, volunteer_entries::Column::StartedAt, volunteer_entries::Column::EndedAt, volunteer_entries::Column::WorkDescription]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into(), Expr::value(p7.clone()).into(), Expr::value(p8.clone()).into()]);
    query.returning(Query::returning().column(volunteer_entries::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn transition_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::TRANSITION_SELECT_USERS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}
pub async fn approver_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {let statement=Statement::from_sql_and_values(DbBackend::Postgres,postgres::APPROVER_SELECT_USERS,vec![p1.into()]);db.query_one(statement).await?.ok_or_else(||DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)}

pub async fn transition_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_entries::Entity);
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::VolunteerUserId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::DepartmentId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Status)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn transition_update_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::TRANSITION_UPDATE_VOLUNTEER_ENTRIES, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn transition_update_volunteer_approval_tokens<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(volunteer_approval_tokens::Entity);
    query.value(volunteer_approval_tokens::Column::RevokedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::EntryId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RedeemedAt))).is_null()).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RevokedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn qr_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
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

pub async fn qr_update_volunteer_approval_tokens<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(volunteer_approval_tokens::Entity);
    query.value(volunteer_approval_tokens::Column::RevokedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::EntryId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RevokedAt))).is_null()).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RedeemedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn qr_insert_volunteer_approval_tokens<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::QR_INSERT_VOLUNTEER_APPROVAL_TOKENS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn preview_select_volunteer_approval_tokens<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<Option<QueryResult>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PREVIEW_SELECT_VOLUNTEER_APPROVAL_TOKENS, vec![p1]);
    db.query_one(statement).await
}

pub async fn decide_select_volunteer_approval_tokens<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_approval_tokens::Entity);
    query.expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::EntryId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::TokenHash))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn decide_select_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_entries::Entity);
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::VolunteerUserId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::DepartmentId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Status)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn decide_select_volunteer_decision_events<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<Option<QueryResult>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(volunteer_decision_events::Entity);
    query.expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::EntryId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::ActorUserId)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::Decision)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::Note)));
    query.expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::TokenHash)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_decision_events::Column::IdempotencyKey))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await
}

pub async fn decide_select_volunteer_approval_tokens_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::DECIDE_SELECT_VOLUNTEER_APPROVAL_TOKENS_2, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn decide_update_volunteer_entries<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::update();
    query.table(volunteer_entries::Entity);
    query.value(volunteer_entries::Column::Status, Expr::value(p2.clone()));
    query.value(volunteer_entries::Column::DecidedAt, Expr::current_timestamp());
    query.value(volunteer_entries::Column::DecidedByUserId, Expr::value(p3.clone()));
    query.value(volunteer_entries::Column::DecisionNote, Expr::value(p4.clone()));
    query.value(volunteer_entries::Column::Version, Expr::col(volunteer_entries::Column::Version).add(1i32));
    query.value(volunteer_entries::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_entries::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn decide_update_volunteer_approval_tokens<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(volunteer_approval_tokens::Entity);
    query.value(volunteer_approval_tokens::Column::RedeemedAt, Expr::current_timestamp());
    query.value(volunteer_approval_tokens::Column::RedeemedByUserId, Expr::value(p2.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::TokenHash))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn decide_update_volunteer_approval_tokens_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(volunteer_approval_tokens::Entity);
    query.value(volunteer_approval_tokens::Column::RevokedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::EntryId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RedeemedAt))).is_null()).add(Expr::expr(SimpleExpr::from(Expr::col(volunteer_approval_tokens::Column::RevokedAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn decide_insert_volunteer_decision_events<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let mut query = Query::insert();
    query.into_table(volunteer_decision_events::Entity).columns([volunteer_decision_events::Column::EntryId, volunteer_decision_events::Column::ActorUserId, volunteer_decision_events::Column::Decision, volunteer_decision_events::Column::Note, volunteer_decision_events::Column::Method, volunteer_decision_events::Column::IdempotencyKey, volunteer_decision_events::Column::TokenHash]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into(), Expr::value(p7.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn report_select_volunteer_entries<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REPORT_SELECT_VOLUNTEER_ENTRIES, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
