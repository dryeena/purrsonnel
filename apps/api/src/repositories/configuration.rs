//! configuration persistence. Callers own workflow transactions and permission checks.
#![allow(unused_imports)]
use sea_orm::{ConnectionTrait, DbBackend, DbErr, ExecResult, QueryResult, Statement, TryGetable, Value};
use sea_orm::sea_query::{Alias, Condition, Expr, Func, LockType, OnConflict, Order, Query, SimpleExpr};
use crate::entities::*;
mod postgres {
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GRANTS_SELECT_PERMISSION_GRANTS: &str = "SELECT to_jsonb(g)||jsonb_build_object('display_name',u.display_name) FROM permission_grants g JOIN users u ON u.id=g.user_id ORDER BY u.display_name,g.permission";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GRANT_SELECT_DEPARTMENTS: &str = "SELECT EXISTS(SELECT 1 FROM departments WHERE id=$1 AND active)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const GRANT_SELECT_DIVISIONS: &str = "SELECT EXISTS(SELECT 1 FROM divisions WHERE id=$1 AND active)";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REVOKE_SELECT_PERMISSION_GRANTS: &str = "SELECT user_id,permission,scope_type,to_jsonb(g) AS data FROM permission_grants g WHERE id=$1 FOR UPDATE";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const REVOKE_SELECT_PERMISSION_GRANTS_2: &str = "SELECT count(*) FROM permission_grants g JOIN users u ON u.id=g.user_id WHERE permission='settings.manage' AND scope_type='global' AND u.status='active' AND g.id<>$1";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const PROFILE_SELECT_PROFILE_FIELD_DEFINITIONS: &str = "SELECT jsonb_build_object('id',f.id,'key',f.key,'label',f.label,'type',f.type,'choices',f.choices_json,'editable_by',f.editable_by,'max_length',f.validation_json->'max_length','value',v.value_json) FROM profile_field_definitions f LEFT JOIN profile_field_values v ON v.definition_id=f.id AND v.user_id=$1 WHERE f.active AND (f.visibility<>'sensitive' OR $2 OR $3) AND (f.visibility<>'self' OR $3 OR $2) ORDER BY f.display_order,f.label";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const TEMPLATES_SELECT_FORM_TEMPLATE_VERSIONS: &str = "SELECT to_jsonb(t)||jsonb_build_object('versions',COALESCE((SELECT jsonb_agg(to_jsonb(v) ORDER BY version_number DESC) FROM form_template_versions v WHERE v.template_id=t.id),'[]')) FROM form_templates t WHERE has_permission($1,'positions.manage',t.department_id) ORDER BY t.name";
/// PostgreSQL-specific policy, projection, conflict handling, or locking workflow.
pub(super) const TEMPLATE_VERSION_INSERT_FORM_TEMPLATE_VERSIONS: &str = "INSERT INTO form_template_versions(template_id,version_number,schema_json) SELECT $1,COALESCE(max(version_number),0)+1,$2 FROM form_template_versions WHERE template_id=$1 RETURNING id";
}

pub async fn grants_select_permission_grants<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GRANTS_SELECT_PERMISSION_GRANTS, vec![]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn grant_select_departments<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GRANT_SELECT_DEPARTMENTS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn grant_select_divisions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::GRANT_SELECT_DIVISIONS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn grant_insert_permission_grants<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>, p5: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let p5: Value = p5.into();
    let mut query = Query::insert();
    query.into_table(permission_grants::Entity).columns([permission_grants::Column::UserId, permission_grants::Column::Permission, permission_grants::Column::ScopeType, permission_grants::Column::ScopeId, permission_grants::Column::GrantedBy]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into(), Expr::value(p5.clone()).into()]);
    query.returning(Query::returning().column(permission_grants::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn revoke_select_policy<C: ConnectionTrait>(db: &C) -> Result<ExecResult, DbErr> {

    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("pg_advisory_xact_lock")).args([Expr::value(512512i32)])));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn revoke_select_permission_grants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REVOKE_SELECT_PERMISSION_GRANTS, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn revoke_select_permission_grants_2<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::REVOKE_SELECT_PERMISSION_GRANTS_2, vec![p1]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn revoke_delete_permission_grants<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::delete();
    query.from_table(permission_grants::Entity).cond_where(Expr::expr(SimpleExpr::from(Expr::col(permission_grants::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn profile_select_policy<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::select();
    query.expr(SimpleExpr::from(Func::cust(Alias::new("visible_person")).args([Expr::value(p1.clone()), Expr::value(p2.clone()), Expr::value("people.read_sensitive")])));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn profile_select_profile_field_definitions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::PROFILE_SELECT_PROFILE_FIELD_DEFINITIONS, vec![p1, p2, p3]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn update_profile_select_profile_field_definitions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<QueryResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(profile_field_definitions::Entity);
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Key)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Label)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Type)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::ChoicesJson)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Visibility)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::EditableBy)));
    query.expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::ValidationJson)));
    query.cond_where(Condition::all().add(Expr::expr(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Id))).eq(Expr::value(p1.clone()))).add(SimpleExpr::from(Expr::col(profile_field_definitions::Column::Active))));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))
}

pub async fn update_profile_insert_profile_field_values<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>, p4: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let p4: Value = p4.into();
    let mut query = Query::insert();
    query.into_table(profile_field_values::Entity).columns([profile_field_values::Column::UserId, profile_field_values::Column::DefinitionId, profile_field_values::Column::ValueJson, profile_field_values::Column::UpdatedByUserId]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into(), Expr::value(p4.clone()).into()]);
    query.on_conflict(OnConflict::columns([profile_field_values::Column::UserId, profile_field_values::Column::DefinitionId]).value(profile_field_values::Column::ValueJson, Expr::value(p3.clone())).value(profile_field_values::Column::UpdatedByUserId, Expr::value(p4.clone())).value(profile_field_values::Column::UpdatedAt, Expr::current_timestamp()).to_owned());
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn update_profile_update_users<C: ConnectionTrait>(db: &C, p1: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::update();
    query.table(users::Entity);
    query.value(users::Column::UpdatedAt, Expr::current_timestamp());
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(users::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn create_level_insert_personnel_levels<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(personnel_levels::Entity).columns([personnel_levels::Column::Label, personnel_levels::Column::Slug, personnel_levels::Column::Rank]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(personnel_levels::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn map_level_select_position_levels<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(position_levels::Entity);
    query.expr(SimpleExpr::from(Expr::col(position_levels::Column::PersonnelLevelId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_levels::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn map_level_update_position_levels<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::update();
    query.table(position_levels::Entity);
    query.value(position_levels::Column::PersonnelLevelId, Expr::value(p2.clone()));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(position_levels::Column::Id))).eq(Expr::value(p1.clone())));
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn templates_select_form_template_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<Vec<T>, DbErr> {
    let p1: Value = p1.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::TEMPLATES_SELECT_FORM_TEMPLATE_VERSIONS, vec![p1]);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_template_select_organization_default_questions<C: ConnectionTrait, T: TryGetable>(db: &C) -> Result<Vec<T>, DbErr> {

    let mut query = Query::select();
    query.from(organization_default_questions::Entity);
    query.expr(SimpleExpr::from(Func::cust(Alias::new("jsonb_build_object")).args([Expr::value("key"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Key)), Expr::value("label"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Label)), Expr::value("type"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Type)), Expr::value("required"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Required)), Expr::value("sensitive"), SimpleExpr::from(Expr::col(organization_default_questions::Column::Sensitive)), Expr::value("max_length"), SimpleExpr::from(Expr::col(organization_default_questions::Column::MaxLength))])));
    query.order_by(organization_default_questions::Column::SortOrder, Order::Asc);
    let statement = db.get_database_backend().build(&query);
    db.query_all(statement).await?.into_iter().map(|row| row.try_get_by_index(0)).collect()
}

pub async fn create_template_insert_form_templates<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>, p3: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let p3: Value = p3.into();
    let mut query = Query::insert();
    query.into_table(form_templates::Entity).columns([form_templates::Column::Name, form_templates::Column::DepartmentId, form_templates::Column::CreatedBy]).values_panic([Expr::value(p1.clone()).into(), Expr::value(p2.clone()).into(), Expr::value(p3.clone()).into()]);
    query.returning(Query::returning().column(form_templates::Column::Id));
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn create_template_insert_form_template_versions<C: ConnectionTrait>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<ExecResult, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let mut query = Query::insert();
    query.into_table(form_template_versions::Entity).columns([form_template_versions::Column::TemplateId, form_template_versions::Column::VersionNumber, form_template_versions::Column::SchemaJson]).values_panic([Expr::value(p1.clone()).into(), Expr::value(1i32).into(), Expr::value(p2.clone()).into()]);
    let statement = db.get_database_backend().build(&query);
    db.execute(statement).await
}

pub async fn template_version_select_form_templates<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let mut query = Query::select();
    query.from(form_templates::Entity);
    query.expr(SimpleExpr::from(Expr::col(form_templates::Column::DepartmentId)));
    query.cond_where(Expr::expr(SimpleExpr::from(Expr::col(form_templates::Column::Id))).eq(Expr::value(p1.clone())));
    query.lock(LockType::Update);
    let statement = db.get_database_backend().build(&query);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}

pub async fn template_version_insert_form_template_versions<C: ConnectionTrait, T: TryGetable>(db: &C, p1: impl Into<Value>, p2: impl Into<Value>) -> Result<T, DbErr> {
    let p1: Value = p1.into();
    let p2: Value = p2.into();
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, postgres::TEMPLATE_VERSION_INSERT_FORM_TEMPLATE_VERSIONS, vec![p1, p2]);
    db.query_one(statement).await?.ok_or_else(|| DbErr::RecordNotFound("Record not found".into()))?.try_get_by_index(0)
}
