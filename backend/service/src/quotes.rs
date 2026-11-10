use crate::error::{AppError, AppResult};
use crate::jobs::access;
use crate::providers::profile_for_user;
use crate::workflow::{self, Actor};
use chrono::{NaiveDate, Utc};
use entity::{job_request, provider_profile, quote, user, JobStatus, QuoteStatus, UserRole, VerificationStatus};
use sea_orm::{
    sea_query::Expr, ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct QuoteInput {
    pub price: f64,
    pub estimated_hours: f64,
    pub earliest_start: NaiveDate,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuoteDto {
    pub id: Uuid,
    pub job_request_id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_avg_rating: f64,
    pub provider_completed_jobs: i32,
    pub price: f64,
    pub estimated_hours: f64,
    pub earliest_start: NaiveDate,
    pub message: Option<String>,
    pub status: QuoteStatus,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

fn to_dto(q: quote::Model, p: &provider_profile::Model) -> QuoteDto {
    QuoteDto {
        id: q.id,
        job_request_id: q.job_request_id,
        provider_id: p.id,
        provider_name: p.business_name.clone(),
        provider_avg_rating: crate::f(p.avg_rating),
        provider_completed_jobs: p.completed_jobs,
        price: crate::f(q.price),
        estimated_hours: crate::f(q.estimated_hours),
        earliest_start: q.earliest_start,
        message: q.message,
        status: q.status,
        created_at: q.created_at,
    }
}

pub fn validate(input: &QuoteInput, today: NaiveDate) -> AppResult<()> {
    if !(input.price > 0.0 && input.price <= 100_000.0) {
        return Err(AppError::field("price", "Cena mora biti večja od 0 in največ 100.000 €."));
    }
    if !(0.5..=200.0).contains(&input.estimated_hours) {
        return Err(AppError::field("estimated_hours", "Trajanje mora biti od 0,5 do 200 ur."));
    }
    if input.earliest_start < today {
        return Err(AppError::field("earliest_start", "Začetek ne sme biti v preteklosti."));
    }
    if input.message.as_ref().is_some_and(|m| m.chars().count() > 2000) {
        return Err(AppError::field("message", "Sporočilo je predolgo."));
    }
    Ok(())
}

fn clean_message(m: Option<String>) -> Option<String> {
    m.map(|m| m.trim().to_string()).filter(|m| !m.is_empty())
}

fn hours(v: f64) -> rust_decimal::Decimal {
    crate::dec(v).round_dp(1)
}

pub async fn create(db: &DatabaseConnection, u: &user::Model, job_id: Uuid, input: QuoteInput) -> AppResult<QuoteDto> {
    let p = profile_for_user(db, u).await?;
    if p.verification_status != VerificationStatus::Verified {
        return Err(AppError::new(
            403,
            "PROVIDER_NOT_VERIFIED",
            "Vaš profil še čaka na preverjanje. Ponudb še ne morete oddati.",
        ));
    }
    validate(&input, crate::today())?;
    let txn = db.begin().await?;
    let a = access(&txn, u, job_id, true).await?;
    if !a.provider_matches {
        return Err(AppError::forbidden("To povpraševanje ni iz vaših kategorij ali območja."));
    }
    if !matches!(a.job.status, JobStatus::Open | JobStatus::Quoted) {
        return Err(AppError::new(
            409,
            "JOB_CLOSED",
            "Na to povpraševanje ni več mogoče oddati ponudbe.",
        ));
    }
    let exists = quote::Entity::find()
        .filter(quote::Column::JobRequestId.eq(job_id))
        .filter(quote::Column::ProviderId.eq(p.id))
        .one(&txn)
        .await?;
    if exists.is_some() {
        return Err(quote_exists());
    }
    let now = Utc::now().fixed_offset();
    let q = quote::ActiveModel {
        id: Set(Uuid::new_v4()),
        job_request_id: Set(job_id),
        provider_id: Set(p.id),
        price: Set(crate::dec(input.price)),
        estimated_hours: Set(hours(input.estimated_hours)),
        earliest_start: Set(input.earliest_start),
        message: Set(clean_message(input.message)),
        status: Set(QuoteStatus::Pending),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            quote_exists()
        } else {
            e.into()
        }
    })?;
    if a.job.status == JobStatus::Open {
        workflow::apply(&txn, a.job, JobStatus::Quoted, Actor::QuotingProvider, u.id, None, None).await?;
    }
    txn.commit().await?;
    Ok(to_dto(q, &p))
}

fn quote_exists() -> AppError {
    AppError::new(409, "QUOTE_EXISTS", "Na to povpraševanje ste že oddali ponudbo.")
}

async fn own_pending_quote(
    db: &impl sea_orm::ConnectionTrait,
    u: &user::Model,
    quote_id: Uuid,
) -> AppResult<(quote::Model, provider_profile::Model)> {
    let p = profile_for_user(db, u).await?;
    let q = quote::Entity::find_by_id(quote_id)
        .lock_exclusive()
        .one(db)
        .await?
        .ok_or_else(AppError::not_found)?;
    if q.provider_id != p.id {
        return Err(AppError::forbidden("To ni vaša ponudba."));
    }
    if q.status != QuoteStatus::Pending {
        return Err(AppError::new(
            409,
            "QUOTE_NOT_PENDING",
            "Ponudbo lahko spremenite samo, dokler čaka na odločitev.",
        ));
    }
    Ok((q, p))
}

pub async fn update(db: &DatabaseConnection, u: &user::Model, quote_id: Uuid, input: QuoteInput) -> AppResult<QuoteDto> {
    validate(&input, crate::today())?;
    let txn = db.begin().await?;
    let (q, p) = own_pending_quote(&txn, u, quote_id).await?;
    let mut am: quote::ActiveModel = q.into();
    am.price = Set(crate::dec(input.price));
    am.estimated_hours = Set(hours(input.estimated_hours));
    am.earliest_start = Set(input.earliest_start);
    am.message = Set(clean_message(input.message));
    am.updated_at = Set(Utc::now().fixed_offset());
    let q = am.update(&txn).await?;
    txn.commit().await?;
    Ok(to_dto(q, &p))
}

pub async fn withdraw(db: &DatabaseConnection, u: &user::Model, quote_id: Uuid) -> AppResult<QuoteDto> {
    let txn = db.begin().await?;
    let (q, p) = own_pending_quote(&txn, u, quote_id).await?;
    let mut am: quote::ActiveModel = q.into();
    am.status = Set(QuoteStatus::Withdrawn);
    am.updated_at = Set(Utc::now().fixed_offset());
    let q = am.update(&txn).await?;
    txn.commit().await?;
    Ok(to_dto(q, &p))
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum QuoteSort {
    #[default]
    Price,
    Start,
}

pub async fn list_for_job(db: &DatabaseConnection, u: &user::Model, job_id: Uuid, sort: QuoteSort) -> AppResult<Vec<QuoteDto>> {
    let job = job_request::Entity::find_by_id(job_id)
        .one(db)
        .await?
        .ok_or_else(AppError::not_found)?;
    if !(u.role == UserRole::Customer && job.customer_id == u.id) && u.role != UserRole::Admin {
        return Err(AppError::forbidden(
            "Ponudbe lahko vidi samo stranka, ki je objavila povpraševanje.",
        ));
    }
    let rows = quote::Entity::find()
        .filter(quote::Column::JobRequestId.eq(job_id))
        .filter(quote::Column::Status.ne(QuoteStatus::Withdrawn))
        .find_also_related(provider_profile::Entity)
        .all(db)
        .await?;
    let mut out: Vec<QuoteDto> = rows.into_iter().filter_map(|(q, p)| p.map(|p| to_dto(q, &p))).collect();
    match sort {
        QuoteSort::Price => out.sort_by(|a, b| a.price.total_cmp(&b.price).then(a.earliest_start.cmp(&b.earliest_start))),
        QuoteSort::Start => out.sort_by(|a, b| a.earliest_start.cmp(&b.earliest_start).then(a.price.total_cmp(&b.price))),
    }
    Ok(out)
}

pub async fn my_quote(db: &DatabaseConnection, u: &user::Model, job_id: Uuid) -> AppResult<Option<QuoteDto>> {
    let p = profile_for_user(db, u).await?;
    let q = quote::Entity::find()
        .filter(quote::Column::JobRequestId.eq(job_id))
        .filter(quote::Column::ProviderId.eq(p.id))
        .one(db)
        .await?;
    Ok(q.map(|q| to_dto(q, &p)))
}

/// Accept one quote: all in one transaction, see US-05.
pub async fn accept(db: &DatabaseConnection, u: &user::Model, quote_id: Uuid) -> AppResult<QuoteDto> {
    let txn = db.begin().await?;
    let q = quote::Entity::find_by_id(quote_id)
        .one(&txn)
        .await?
        .ok_or_else(AppError::not_found)?;
    let job = job_request::Entity::find_by_id(q.job_request_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or_else(AppError::not_found)?;
    if !(u.role == UserRole::Customer && job.customer_id == u.id) {
        return Err(AppError::forbidden(
            "Ponudbo lahko sprejme samo stranka, ki je objavila povpraševanje.",
        ));
    }
    workflow::check_transition(Some(job.status), JobStatus::Accepted, Actor::OwnerCustomer)?;
    let q = quote::Entity::find_by_id(quote_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or_else(AppError::not_found)?;
    if q.status != QuoteStatus::Pending {
        return Err(AppError::new(409, "QUOTE_NOT_PENDING", "Te ponudbe ni več mogoče sprejeti."));
    }
    let now = Utc::now().fixed_offset();
    quote::Entity::update_many()
        .col_expr(quote::Column::Status, QuoteStatus::Rejected.as_enum())
        .col_expr(quote::Column::UpdatedAt, Expr::value(now))
        .filter(quote::Column::JobRequestId.eq(job.id))
        .filter(quote::Column::Status.eq(QuoteStatus::Pending))
        .filter(quote::Column::Id.ne(q.id))
        .exec(&txn)
        .await?;
    let mut am: quote::ActiveModel = q.into();
    am.status = Set(QuoteStatus::Accepted);
    am.updated_at = Set(now);
    let q = am.update(&txn).await?;
    workflow::apply(&txn, job, JobStatus::Accepted, Actor::OwnerCustomer, u.id, None, Some(q.id)).await?;
    let p = provider_profile::Entity::find_by_id(q.provider_id)
        .one(&txn)
        .await?
        .ok_or_else(AppError::not_found)?;
    txn.commit().await?;
    Ok(to_dto(q, &p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> QuoteInput {
        QuoteInput {
            price: 60.0,
            estimated_hours: 1.0,
            earliest_start: NaiveDate::from_ymd_opt(2026, 11, 10).unwrap(),
            message: None,
        }
    }

    #[test]
    fn limits() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        assert!(validate(&input(), today).is_ok());
        for (price, field) in [(0.0, "price"), (100_000.01, "price")] {
            let mut i = input();
            i.price = price;
            assert_eq!(validate(&i, today).unwrap_err().field.as_deref(), Some(field));
        }
        let mut i = input();
        i.price = 100_000.0;
        assert!(validate(&i, today).is_ok());
        for h in [0.4, 200.5] {
            let mut i = input();
            i.estimated_hours = h;
            assert_eq!(validate(&i, today).unwrap_err().field.as_deref(), Some("estimated_hours"));
        }
        let mut i = input();
        i.earliest_start = NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        assert_eq!(validate(&i, today).unwrap_err().field.as_deref(), Some("earliest_start"));
        i.earliest_start = today;
        assert!(validate(&i, today).is_ok());
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyQuoteItem {
    pub quote: QuoteDto,
    pub job_title: String,
    pub job_status: JobStatus,
    pub job_city: String,
}

/// Provider's own quotes, so he can find accepted jobs again.
pub async fn list_mine(db: &DatabaseConnection, u: &user::Model) -> AppResult<Vec<MyQuoteItem>> {
    use sea_orm::QueryOrder;
    let p = profile_for_user(db, u).await?;
    let rows = quote::Entity::find()
        .filter(quote::Column::ProviderId.eq(p.id))
        .order_by_desc(quote::Column::CreatedAt)
        .find_also_related(job_request::Entity)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(q, j)| {
            j.map(|j| MyQuoteItem {
                quote: to_dto(q, &p),
                job_title: j.title,
                job_status: j.status,
                job_city: j.city,
            })
        })
        .collect())
}
