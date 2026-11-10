//! Entity-backed directory filtering, paging and sorting. No SQL string assembly.
use sea_orm::{ColumnTrait,Condition,ConnectionTrait,DbErr,EntityTrait,PaginatorTrait,QueryFilter,QueryOrder,QuerySelect,Select};
use sea_orm::sea_query::{Alias,Expr,Func,JoinType,Order,Query,SimpleExpr};
use sea_orm::sea_query::extension::postgres::PgExpr;
use serde::Deserialize;
use uuid::Uuid;
use crate::entities::{people_directory as d,user_departments as m,departments as dep,user_service_years as y,manager_relationships as r,term_versions as v,term_signatures as sig};

#[derive(Deserialize,Default)]pub struct Filter{
 pub q:Option<String>,pub department:Option<Uuid>,pub division:Option<Uuid>,pub level:Option<Uuid>,
 pub status:Option<String>,pub volunteer:Option<bool>,pub year:Option<i32>,pub skill:Option<String>,
 pub manager:Option<Uuid>,pub pending_terms:Option<bool>,pub sort:Option<String>,pub order:Option<String>,pub page:Option<i64>,pub page_size:Option<i64>,
}
pub fn select(actor:Uuid,f:&Filter)->Select<d::Entity>{
 let scope=Func::cust(Alias::new("visible_person")).args([Expr::value(actor),Expr::col((d::Entity,d::Column::Id)).into(),Expr::value("people.read")]);
 let mut find=d::Entity::find().filter(Condition::any().add(d::Column::Id.eq(actor)).add(Expr::expr(scope).eq(true)));
 if let Some(text)=&f.q{
  let pattern=format!("%{}%",text.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_"));
  let custom=Expr::cust_with_values("EXISTS(SELECT 1 FROM profile_field_values pfv JOIN profile_field_definitions pfd ON pfd.id=pfv.definition_id WHERE pfv.user_id=people_directory.id AND pfd.active AND pfd.show_on_people_page AND pfv.value_json::text ILIKE $1)",[pattern.clone()]);
  find=find.filter(Condition::any().add(Expr::col(d::Column::DisplayName).ilike(pattern.clone())).add(Expr::col(d::Column::MyfwaId).ilike(pattern)).add(custom));
 }
 if let Some(id)=f.department{find=find.filter(d::Column::Id.in_subquery(Query::select().column(m::Column::UserId).from(m::Entity).and_where(m::Column::EndedAt.is_null()).and_where(m::Column::DepartmentId.eq(id)).to_owned()));}
 if let Some(id)=f.division{find=find.filter(d::Column::Id.in_subquery(Query::select().column(m::Column::UserId).from(m::Entity).join(JoinType::InnerJoin,dep::Entity,Expr::col((m::Entity,m::Column::DepartmentId)).equals((dep::Entity,dep::Column::Id))).and_where(m::Column::EndedAt.is_null()).and_where(dep::Column::DivisionId.eq(id)).to_owned()));}
 if let Some(id)=f.level{find=find.filter(d::Column::PersonnelLevelId.eq(id));}
 if let Some(status)=&f.status{find=find.filter(d::Column::Status.eq(status.clone()));}
 if let Some(value)=f.volunteer{find=find.filter(d::Column::VolunteerEligible.eq(value));}
 if let Some(year)=f.year{find=find.filter(d::Column::Id.in_subquery(Query::select().column(y::Column::UserId).from(y::Entity).and_where(y::Column::VerificationStatus.eq("verified")).and_where(y::Column::Year.eq(year)).to_owned()));}
 if let Some(skill)=&f.skill{find=find.filter(Expr::expr(Func::cust(Alias::new("jsonb_exists")).args([Expr::col(d::Column::Skills).into(),Expr::value(skill.clone())])).eq(true));}
 if let Some(id)=f.manager{find=find.filter(d::Column::Id.in_subquery(Query::select().column(r::Column::DirectUserId).from(r::Entity).and_where(r::Column::EndedAt.is_null()).and_where(r::Column::ManagerUserId.eq(id)).to_owned()));}
 if let Some(pending)=f.pending_terms{
  let signed=Query::select().expr(Expr::value(1)).from(sig::Entity).and_where(Expr::col((sig::Entity,sig::Column::UserId)).equals((d::Entity,d::Column::Id))).and_where(Expr::col((sig::Entity,sig::Column::TermVersionId)).equals((v::Entity,v::Column::Id))).to_owned();
  let applicable=Func::cust(Alias::new("applicable_term")).args([Expr::col((v::Entity,v::Column::TermId)).into(),Expr::col((d::Entity,d::Column::Id)).into()]);
  let outstanding=Query::select().expr(Expr::value(1)).from(v::Entity).and_where(v::Column::PublishedAt.is_not_null()).and_where(v::Column::SupersededAt.is_null()).and_where(Expr::expr(applicable).eq(true)).and_where(Expr::expr(Expr::exists(signed)).not()).to_owned();
  let condition:SimpleExpr=Expr::exists(outstanding);find=find.filter(if pending{condition}else{Expr::expr(condition).not()});
 }
 let column=match f.sort.as_deref(){Some("verified_years")=>d::Column::VerifiedYears,Some("department")=>d::Column::Department,Some("level")=>d::Column::Level,Some("status")=>d::Column::Status,_=>d::Column::DisplayName};
 find.order_by(column,if f.order.as_deref()==Some("desc"){Order::Desc}else{Order::Asc}).order_by_asc(d::Column::Id)
}
pub async fn list<C:ConnectionTrait>(db:&C,actor:Uuid,f:&Filter)->Result<(Vec<serde_json::Value>,u64),DbErr>{
 let query=select(actor,f);let total=query.clone().count(db).await?;
 let limit=f.page_size.unwrap_or(25).clamp(1,100) as u64;let offset=(f.page.unwrap_or(1).clamp(1,100000) as u64-1)*limit;
 Ok((query.limit(limit).offset(offset).into_json().all(db).await?,total))
}
