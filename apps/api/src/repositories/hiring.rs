//! hiring persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const POSITIONS_SELECT_APPLICATIONS: &str = "SELECT to_jsonb(pos)||jsonb_build_object('department',d.name,'level',l.label,'questions',f.schema_json,'owner_name',u.display_name,'application_count',CASE WHEN has_permission($1,'applications.review',pos.department_id) THEN (SELECT count(*) FROM applications a WHERE a.position_posting_id=pos.id) ELSE NULL END) FROM position_postings pos JOIN departments d ON d.id=pos.department_id JOIN position_levels l ON l.id=pos.position_level_id JOIN form_template_versions f ON f.id=pos.form_template_version_id JOIN users u ON u.id=pos.hiring_owner_user_id WHERE pos.status='open' OR has_permission($1,'positions.manage',pos.department_id) ORDER BY pos.created_at DESC LIMIT $2 OFFSET $3";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CREATE_POSITION_SELECT_FORM_TEMPLATE_VERSIONS: &str = "SELECT v.schema_json FROM form_template_versions v JOIN form_templates t ON t.id=v.template_id WHERE v.id=$1 AND t.active AND (t.department_id IS NULL OR t.department_id=$2)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const CREATE_POSITION_INSERT_POSITION_POSTINGS: &str = "INSERT INTO position_postings(title,department_id,position_level_id,is_leadership,description_markdown,hiring_owner_user_id,status,created_by_user_id,form_template_version_id,opened_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,CASE WHEN $7='open' THEN now() ELSE NULL END) RETURNING id";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const APPLICATIONS_SELECT_APPLICATIONS: &str = "SELECT (to_jsonb(a)-'answers_json')||jsonb_build_object('applicant_name',u.display_name,'position',pos.title,'department',d.name,'department_id',d.id,'hiring_owner_user_id',pos.hiring_owner_user_id) FROM applications a JOIN users u ON u.id=a.applicant_user_id JOIN position_postings pos ON pos.id=a.position_posting_id JOIN departments d ON d.id=pos.department_id WHERE a.applicant_user_id=$1 OR has_permission($1,'applications.review',pos.department_id) ORDER BY a.created_at DESC LIMIT $2 OFFSET $3";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const APPLICATION_SELECT_APPLICATIONS: &str = "SELECT a.applicant_user_id,pos.department_id,f.schema_json,to_jsonb(a)||jsonb_build_object('position',pos.title,'questions',f.schema_json) AS data FROM applications a JOIN position_postings pos ON pos.id=a.position_posting_id JOIN form_template_versions f ON f.id=pos.form_template_version_id WHERE a.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SUBMIT_APPLICATION_SELECT_APPLICATIONS: &str = "SELECT a.applicant_user_id,a.status,a.answers_json,f.schema_json,pos.status AS posting_status,pos.closes_at,pos.department_id FROM applications a JOIN position_postings pos ON pos.id=a.position_posting_id JOIN form_template_versions f ON f.id=pos.form_template_version_id WHERE a.id=$1 FOR UPDATE OF a FOR SHARE OF pos";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SUBMIT_APPLICATION_UPDATE_APPLICATIONS: &str = "UPDATE applications SET status='submitted',submitted_at=now(),updated_at=now(),submitted_myfwa_id=(SELECT myfwa_id FROM users WHERE id=$2) WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const INTERVIEWS_SELECT_INTERVIEW_PARTICIPANTS: &str = "SELECT to_jsonb(i)||jsonb_build_object('applicant_name',u.display_name,'applicant_user_id',a.applicant_user_id,'position',pos.title,'department_id',pos.department_id,'hiring_owner_user_id',pos.hiring_owner_user_id,'application_status',a.status,'panel',COALESCE((SELECT jsonb_agg(jsonb_build_object('user_id',ip.interviewer_user_id,'display_name',pu.display_name,'response',ip.response)) FROM interview_participants ip JOIN users pu ON pu.id=ip.interviewer_user_id WHERE ip.interview_id=i.id),'[]')) FROM interviews i JOIN applications a ON a.id=i.application_id JOIN users u ON u.id=a.applicant_user_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE a.applicant_user_id=$1 OR has_permission($1,'interviews.manage',pos.department_id) OR EXISTS(SELECT 1 FROM interview_participants ip WHERE ip.interview_id=i.id AND ip.interviewer_user_id=$1) ORDER BY i.scheduled_start_at LIMIT $2 OFFSET $3";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SCHEDULE_SELECT_APPLICATIONS: &str = "SELECT pos.department_id FROM applications a JOIN position_postings pos ON pos.id=a.position_posting_id WHERE a.id=$1 AND a.status IN ('submitted','screening','interviewing')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const SCHEDULE_SELECT_PG_TIMEZONE_NAMES: &str = "SELECT EXISTS(SELECT 1 FROM pg_timezone_names WHERE name=$1)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RECOMMEND_SELECT_INTERVIEWS: &str = "SELECT pos.department_id FROM interviews i JOIN applications a ON a.id=i.application_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE i.id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const RECOMMEND_UPDATE_INTERVIEW_PARTICIPANTS: &str = "UPDATE interview_participants ip SET response=$3,response_at=now(),recommendation_notes=$4 FROM interviews i,applications a WHERE ip.interview_id=$1 AND ip.interviewer_user_id=$2 AND i.id=ip.interview_id AND a.id=i.application_id AND a.status='interviewing' AND i.status IN ('scheduled','completed')";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const NOTES_SELECT_INTERVIEW_NOTES: &str = "SELECT to_jsonb(n)||jsonb_build_object('author_name',u.display_name) FROM interview_notes n JOIN users u ON u.id=n.author_user_id JOIN interviews i ON i.id=n.interview_id JOIN applications a ON a.id=i.application_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE n.interview_id=$1 AND (n.author_user_id=$2 OR (n.visibility='interview_panel' AND EXISTS(SELECT 1 FROM interview_participants ip WHERE ip.interview_id=i.id AND ip.interviewer_user_id=$2)) OR (n.visibility='hr' AND has_permission($2,'people.read_sensitive',pos.department_id))) ORDER BY n.created_at";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const ADD_NOTE_SELECT_INTERVIEWS: &str = "SELECT EXISTS(SELECT 1 FROM interviews i JOIN applications a ON a.id=i.application_id JOIN position_postings pos ON pos.id=a.position_posting_id WHERE i.id=$1 AND (has_permission($2,'interviews.manage',pos.department_id) OR EXISTS(SELECT 1 FROM interview_participants ip WHERE ip.interview_id=i.id AND ip.interviewer_user_id=$2)))";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const FINALIZE_SELECT_APPLICATIONS: &str = "SELECT a.status,a.applicant_user_id,pos.department_id,pos.hiring_owner_user_id,l.personnel_level_id FROM applications a JOIN position_postings pos ON pos.id=a.position_posting_id JOIN position_levels l ON l.id=pos.position_level_id WHERE a.id=$1 FOR UPDATE OF a FOR SHARE OF pos,l";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const FINALIZE_UPDATE_USERS: &str = "UPDATE users SET personnel_level_id=CASE WHEN COALESCE((SELECT rank FROM personnel_levels WHERE id=users.personnel_level_id),0)>(SELECT rank FROM personnel_levels WHERE id=$2) THEN personnel_level_id ELSE $2 END,volunteer_eligible=false,updated_at=now() WHERE id=$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const FINALIZE_INSERT_USER_DEPARTMENTS: &str = "INSERT INTO user_departments(user_id,department_id) VALUES($1,$2) ON CONFLICT(user_id,department_id) WHERE ended_at IS NULL DO NOTHING";
}

pub async fn positions_select_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::POSITIONS_SELECT_APPLICATIONS, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_position_select_organization_default_questions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(organization_default_questions::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("key"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Key)), Expr::value("label"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Label)), Expr::value("type"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Type)), Expr::value("required"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Required)), Expr::value("sensitive"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Sensitive)), Expr::value("max_length"), SimpleExpr::from(Expr::col(organization_default_questions::Column::MaxLength))])));
    query.order_by(organization_default_questions::Column::SortOrder, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_position_select_form_template_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CREATE_POSITION_SELECT_FORM_TEMPLATE_VERSIONS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_position_insert_form_templates<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(form_templates::Entity).columns([form_templates::Column::Name, form_templates::Column::DepartmentId, form_templates::Column::CreatedBy]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(form_templates::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_position_insert_form_template_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(form_template_versions::Entity).columns([form_template_versions::Column::TemplateId, form_template_versions::Column::VersionNumber, form_template_versions::Column::SchemaJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value(1i32).into(), Expr::value(p2.clone()).into()]);
    query.returning(Query::returning().column(form_template_versions::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_position_insert_position_postings<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>, p8: impl Into<Value>, p9: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let p8: Value = p8.into();
    let p9: Value = p9.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::CREATE_POSITION_INSERT_POSITION_POSTINGS, vec![p1, p2, p3, p4, p5, p6, p7, p8, p9]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn update_position_select_position_postings<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(position_postings::Entity);
    query.expr(SimpleExpr::from(Expr::col(position_postings::Column::DepartmentId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn update_position_update_position_postings<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(position_postings::Entity);
    query.value(position_postings::Column::Status, Expr::value(p2.clone()));
    query.value(position_postings::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn questions_select_organization_default_questions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(organization_default_questions::Entity);
    query.expr(Func::cust(Alias::new("to_jsonb")).arg(Expr::col(Alias::new("organization_default_questions"))));
    query.order_by(organization_default_questions::Column::SortOrder, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_question_insert_organization_default_questions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(organization_default_questions::Entity).columns([organization_default_questions::Column::Key, organization_default_questions::Column::Label, organization_default_questions::Column::Type, organization_default_questions::Column::Required, organization_default_questions::Column::Sensitive]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    query.returning(Query::returning().column(organization_default_questions::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn applications_select_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::APPLICATIONS_SELECT_APPLICATIONS, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn apply_select_position_postings<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(position_postings::Entity);
    query.expr(SimpleExpr::from(Expr::col(position_postings::Column::DepartmentId)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::Status))).eq(Expr::value("open"))).add(Condition::any().add(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::ClosesAt))).is_null()).add(Expr::expr(SimpleExpr::from(Expr::col(position_postings::Column::ClosesAt))).gt(Expr::current_timestamp()))));
    query.lock(LockType::Share);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn apply_insert_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(applications::Entity).columns([applications::Column::PositionPostingId, applications::Column::ApplicantUserId, applications::Column::AnswersJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(applications::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn application_select_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::APPLICATION_SELECT_APPLICATIONS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn edit_application_update_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::update();
    query.table(applications::Entity);
    query.value(applications::Column::AnswersJson, Expr::value(p3.clone()));
    query.value(applications::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::Id))).eq(Expr::value(p1.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::ApplicantUserId))).eq(Expr::value(p2.clone()))).add(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::Status))).eq(Expr::value("draft"))));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn submit_application_select_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SUBMIT_APPLICATION_SELECT_APPLICATIONS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn submit_application_update_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SUBMIT_APPLICATION_UPDATE_APPLICATIONS, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn interviews_select_interview_participants<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::INTERVIEWS_SELECT_INTERVIEW_PARTICIPANTS, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn schedule_select_applications<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SCHEDULE_SELECT_APPLICATIONS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_select_pg_timezone_names<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::SCHEDULE_SELECT_PG_TIMEZONE_NAMES, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_select_applications_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(applications::Entity);
    query.expr(SimpleExpr::from(Expr::col(applications::Column::Status)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_insert_interviews<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>, p6: impl Into<Value>, p7: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let p6: Value = p6.into();
    let p7: Value = p7.into();
    let mut query = Query::insert();
    query.into_table(interviews::Entity).columns([interviews::Column::ApplicationId, interviews::Column::ScheduledStartAt, interviews::Column::ScheduledEndAt, interviews::Column::Timezone, interviews::Column::LocationLabel, interviews::Column::LocationUrl, interviews::Column::CreatedBy]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into(), Expr::value(p6.clone()).into(), Expr::value(p7.clone()).into()]);
    query.returning(Query::returning().column(interviews::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn schedule_insert_interview_participants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(interview_participants::Entity).columns([interview_participants::Column::InterviewId, interview_participants::Column::InterviewerUserId, interview_participants::Column::AssignedByUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.on_conflict(OnConflict::new().do_nothing().to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn schedule_update_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(applications::Entity);
    query.value(applications::Column::Status, Expr::value("interviewing"));
    query.value(applications::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn recommend_select_interviews<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RECOMMEND_SELECT_INTERVIEWS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn recommend_update_interview_participants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::RECOMMEND_UPDATE_INTERVIEW_PARTICIPANTS, vec![p1, p2, p3, p4]);
    db.execute(statement).await
}

pub async fn notes_select_interview_notes<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::NOTES_SELECT_INTERVIEW_NOTES, vec![p1, p2]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn add_note_select_interviews<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::ADD_NOTE_SELECT_INTERVIEWS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn add_note_insert_interview_notes<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(interview_notes::Entity).columns([interview_notes::Column::InterviewId, interview_notes::Column::AuthorUserId, interview_notes::Column::Body, interview_notes::Column::Visibility]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    query.returning(Query::returning().column(interview_notes::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn finalize_select_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::FINALIZE_SELECT_APPLICATIONS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn finalize_select_application_final_decisions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<Option<QueryResult>, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(application_final_decisions::Entity);
    query.expr(SimpleExpr::from(Expr::col(application_final_decisions::Column::ApplicationId)));
    query.expr(SimpleExpr::from(Expr::col(application_final_decisions::Column::DecidedByUserId)));
    query.expr(SimpleExpr::from(Expr::col(application_final_decisions::Column::Decision)));
    query.expr(SimpleExpr::from(Expr::col(application_final_decisions::Column::Reason)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(application_final_decisions::Column::IdempotencyKey))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await
}

pub async fn finalize_select_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(users::Entity);
    query.expr(SimpleExpr::from(Expr::col(users::Column::Id)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn finalize_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::FINALIZE_UPDATE_USERS, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn finalize_insert_user_departments<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::FINALIZE_INSERT_USER_DEPARTMENTS, vec![p1, p2]);
    db.execute(statement).await
}

pub async fn finalize_insert_application_final_decisions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(application_final_decisions::Entity).columns([application_final_decisions::Column::ApplicationId, application_final_decisions::Column::Decision, application_final_decisions::Column::DecidedByUserId, application_final_decisions::Column::Reason, application_final_decisions::Column::IdempotencyKey]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn finalize_update_applications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::update();
    query.table(applications::Entity);
    query.value(applications::Column::Status, Expr::value(p2.clone()));
    query.value(applications::Column::DecidedAt, Expr::current_timestamp());
    query.value(applications::Column::DecisionActorId, Expr::value(p3.clone()));
    query.value(applications::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(applications::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn finalize_insert_notifications<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(notifications::Entity).columns([notifications::Column::RecipientUserId, notifications::Column::Type, notifications::Column::PayloadJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value("application_decision").into(), Expr::value(p2.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}
