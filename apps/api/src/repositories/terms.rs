//! terms persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const APPLICABLE_SELECT_TERMS: &str = "SELECT jsonb_build_object('id',t.id,'title',t.title,'audience',t.audience,'version_id',v.id,'version_number',v.version_number,'body_markdown',v.body_markdown,'body_sha256',v.body_sha256,'published_at',v.published_at,'signed_at',sig.consented_at,'typed_name',sig.typed_name) FROM terms t JOIN term_versions v ON v.term_id=t.id AND v.published_at IS NOT NULL AND v.superseded_at IS NULL LEFT JOIN term_signatures sig ON sig.term_version_id=v.id AND sig.user_id=$1 WHERE applicable_term(t.id,$1) ORDER BY t.title";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const LIST_SELECT_TERM_VERSIONS: &str = "SELECT to_jsonb(t)||jsonb_build_object('versions',COALESCE((SELECT jsonb_agg(to_jsonb(v) ORDER BY version_number DESC) FROM term_versions v WHERE term_id=t.id),'[]')) FROM terms t ORDER BY title";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const VERSION_INSERT_TERM_VERSIONS: &str = "INSERT INTO term_versions(term_id,version_number,body_markdown,body_sha256) SELECT $1,COALESCE(max(version_number),0)+1,$2,$3 FROM term_versions WHERE term_id=$1 RETURNING id";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PUBLISH_SELECT_TERM_VERSIONS: &str = "SELECT EXISTS(SELECT 1 FROM term_versions WHERE id=$1 AND term_id=$2 AND published_at IS NULL)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SIGN_INSERT_TERM_SIGNATURES: &str = "INSERT INTO term_signatures(term_version_id,user_id,typed_name,affirmation_text,document_sha256) VALUES($1,$2,$3,'I have read this exact version and agree to acknowledge it.',$4) ON CONFLICT(term_version_id,user_id) DO NOTHING RETURNING to_jsonb(term_signatures)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const COMPLIANCE_SELECT_USERS: &str = "SELECT jsonb_build_object('user_id',u.id,'display_name',u.display_name,'term_id',t.id,'title',t.title,'version_number',v.version_number,'signed_at',sig.consented_at) FROM users u CROSS JOIN terms t JOIN term_versions v ON v.term_id=t.id AND v.published_at IS NOT NULL AND v.superseded_at IS NULL LEFT JOIN term_signatures sig ON sig.term_version_id=v.id AND sig.user_id=u.id WHERE visible_person($1,u.id,'terms.read_compliance') AND applicable_term(t.id,u.id) AND u.status='active' ORDER BY u.display_name,t.title LIMIT 1000";
}

pub async fn applicable_select_terms<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::APPLICABLE_SELECT_TERMS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn list_select_term_versions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::LIST_SELECT_TERM_VERSIONS, vec![]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_insert_terms<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(terms::Entity).columns([terms::Column::Title, terms::Column::Audience, terms::Column::MinimumPersonnelRank, terms::Column::RequiresVolunteer, terms::Column::CreatedBy]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    query.returning(Query::returning().column(terms::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_insert_term_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(term_versions::Entity).columns([term_versions::Column::TermId, term_versions::Column::VersionNumber, term_versions::Column::BodyMarkdown, term_versions::Column::BodySha256]).values_panic([Expr::value(p1.clone()).into(), Expr::value(1i32).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(term_versions::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn version_select_terms<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(terms::Entity);
    query.expr(SimpleExpr::from(Expr::col(terms::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(terms::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn version_insert_term_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::VERSION_INSERT_TERM_VERSIONS, vec![p1, p2, p3]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn publish_select_terms<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(terms::Entity);
    query.expr(SimpleExpr::from(Expr::col(terms::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(terms::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn publish_select_term_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PUBLISH_SELECT_TERM_VERSIONS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn publish_update_term_versions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(term_versions::Entity);
    query.value(term_versions::Column::SupersededAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::TermId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::PublishedAt))).is_not_null()).add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::SupersededAt))).is_null()));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn publish_update_term_versions_2<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(term_versions::Entity);
    query.value(term_versions::Column::PublishedAt, Expr::current_timestamp());
    query.value(term_versions::Column::PublishedBy, Expr::value(p2.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn sign_select_term_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(term_versions::Entity);
    query.expr(SimpleExpr::from(Expr::col(term_versions::Column::TermId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn sign_select_terms<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(terms::Entity);
    query.expr(SimpleExpr::from(Expr::col(terms::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(terms::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Share);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn sign_select_term_versions_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(term_versions::Entity);
    query.expr(SimpleExpr::from(Expr::col(term_versions::Column::BodySha256)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::PublishedAt))).is_not_null()).add(Expr::expr(SimpleExpr::from(Expr::col(term_versions::Column::SupersededAt))).is_null()).add(SimpleExpr::from(Func::cust(Alias::new("applicable_term")).args([SimpleExpr::from(Expr::col(term_versions::Column::TermId)), Expr::value(p2.clone())]))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn sign_insert_term_signatures<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<Option<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SIGN_INSERT_TERM_SIGNATURES, vec![p1, p2, p3, p4]);
    db.query_one(statement).await?.map(|row| row.try_get_by_index(0)).transpose()
}

pub async fn sign_select_term_signatures<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.from(term_signatures::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("term_signatures"))));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(term_signatures::Column::TermVersionId))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(term_signatures::Column::UserId))).eq(Expr::value(p2.clone()))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn compliance_select_users<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::COMPLIANCE_SELECT_USERS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}
