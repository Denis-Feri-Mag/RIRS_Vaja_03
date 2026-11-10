use crate::error::{AppError, AppResult};
use crate::providers::{profile_for_user, valid_postal_code};
use crate::workflow::{self, Actor};
use chrono::{Duration, NaiveDate, Utc};
use entity::{
    job_request, job_status_history, provider_category, provider_profile, quote, service_category, user, JobStatus, QuoteStatus, UserRole,
    VerificationStatus,
};
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub const PAGE_SIZE: u64 = 20;
pub const EXPIRES_DAYS: i64 = 14;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateJobInput {
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub city: String,
    pub postal_code: String,
    pub address_line: Option<String>,
    pub preferred_from: Option<NaiveDate>,
    pub preferred_to: Option<NaiveDate>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContactDto {
    pub full_name: String,
    pub phone: Option<String>,
    pub business_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobDto {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub title: String,
    pub description: String,
    pub city: String,
    pub postal_code: String,
    /// only for the customer and the accepted provider
    pub address_line: Option<String>,
    pub preferred_from: Option<NaiveDate>,
    pub preferred_to: Option<NaiveDate>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub status: JobStatus,
    pub accepted_quote_id: Option<Uuid>,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    /// quotes that are not withdrawn
    pub active_quote_count: i64,
    pub is_owner: bool,
    pub is_accepted_provider: bool,
    /// customer contact for the accepted provider
    pub customer: Option<ContactDto>,
    /// accepted provider contact for the customer
    pub provider: Option<ContactDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobListItem {
    pub id: Uuid,
    pub title: String,
    pub category_name: String,
    pub city: String,
    pub status: JobStatus,
    pub quote_count: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema, FromQueryResult)]
pub struct MatchingItem {
    pub id: Uuid,
    pub title: String,
    pub category_name: String,
    pub city: String,
    pub postal_code: String,
    pub preferred_from: Option<NaiveDate>,
    pub preferred_to: Option<NaiveDate>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub status: String,
    pub quote_count: i64,
    pub quote_sent: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MatchingPage {
    pub items: Vec<MatchingItem>,
    pub page: u64,
    pub page_size: u64,
    pub total: i64,
    pub verification_status: VerificationStatus,
    pub category_names: Vec<String>,
    pub service_postal_codes: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryDto {
    pub from_status: Option<JobStatus>,
    pub to_status: JobStatus,
    pub changed_by_name: String,
    pub changed_by_role: UserRole,
    pub note: Option<String>,
    pub changed_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StatusInput {
    pub status: JobStatus,
    pub note: Option<String>,
}

fn validate(input: &CreateJobInput) -> AppResult<()> {
    let title = input.title.trim().chars().count();
    if !(5..=150).contains(&title) {
        return Err(AppError::field("title", "Naslov mora imeti od 5 do 150 znakov."));
    }
    if input.description.trim().chars().count() < 20 {
        return Err(AppError::field("description", "Opis mora imeti vsaj 20 znakov."));
    }
    let city = input.city.trim().chars().count();
    if city == 0 || city > 100 {
        return Err(AppError::field("city", "Vpišite kraj."));
    }
    if !valid_postal_code(input.postal_code.trim()) {
        return Err(AppError::field("postal_code", "Poštna številka mora imeti 4 števke."));
    }
    if input.address_line.as_ref().is_some_and(|a| a.trim().chars().count() > 200) {
        return Err(AppError::field("address_line", "Ulica je predolga."));
    }
    if let (Some(a), Some(b)) = (input.preferred_from, input.preferred_to) {
        if a > b {
            return Err(AppError::field("preferred_to", "Konec termina ne sme biti pred začetkom."));
        }
    }
    for (name, v) in [("budget_min", input.budget_min), ("budget_max", input.budget_max)] {
        if v.is_some_and(|v| !(0.0..=100_000.0).contains(&v)) {
            return Err(AppError::field(name, "Znesek mora biti med 0 in 100.000 €."));
        }
    }
    if let (Some(a), Some(b)) = (input.budget_min, input.budget_max) {
        if a > b {
            return Err(AppError::field(
                "budget_max",
                "Najnižji znesek mora biti manjši ali enak najvišjemu.",
            ));
        }
    }
    Ok(())
}

pub async fn create(db: &DatabaseConnection, u: &user::Model, input: CreateJobInput) -> AppResult<JobDto> {
    if u.role != UserRole::Customer {
        return Err(AppError::forbidden("Povpraševanje lahko objavi samo stranka."));
    }
    validate(&input)?;
    let cat = service_category::Entity::find_by_id(input.category_id).one(db).await?;
    if !cat.is_some_and(|c| c.is_active) {
        return Err(AppError::field("category_id", "Izberite aktivno kategorijo."));
    }
    workflow::check_transition(None, JobStatus::Open, Actor::OwnerCustomer)?;
    let now = Utc::now().fixed_offset();
    let txn = db.begin().await?;
    let job = job_request::ActiveModel {
        id: Set(Uuid::new_v4()),
        customer_id: Set(u.id),
        category_id: Set(input.category_id),
        title: Set(input.title.trim().to_string()),
        description: Set(input.description.trim().to_string()),
        city: Set(input.city.trim().to_string()),
        postal_code: Set(input.postal_code.trim().to_string()),
        address_line: Set(input.address_line.map(|a| a.trim().to_string()).filter(|a| !a.is_empty())),
        preferred_from: Set(input.preferred_from),
        preferred_to: Set(input.preferred_to),
        budget_min: Set(input.budget_min.map(crate::dec)),
        budget_max: Set(input.budget_max.map(crate::dec)),
        status: Set(JobStatus::Open),
        accepted_quote_id: Set(None),
        expires_at: Set(now + Duration::days(EXPIRES_DAYS)),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&txn)
    .await?;
    job_status_history::ActiveModel {
        job_request_id: Set(job.id),
        from_status: Set(None),
        to_status: Set(JobStatus::Open),
        changed_by: Set(u.id),
        note: Set(None),
        changed_at: Set(now),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    txn.commit().await?;
    get(db, u, job.id).await
}

/// Who the user is for this job.
pub struct JobAccess {
    pub job: job_request::Model,
    pub is_owner: bool,
    pub accepted_provider_user: Option<Uuid>,
    pub provider: Option<provider_profile::Model>,
    pub provider_matches: bool,
}

impl JobAccess {
    pub fn actor(&self, u: &user::Model) -> Actor {
        Actor::resolve(u.role, u.id, &self.job, self.accepted_provider_user)
    }
    pub fn is_accepted_provider(&self, u: &user::Model) -> bool {
        self.accepted_provider_user == Some(u.id)
    }
}

pub async fn accepted_provider_user<C: ConnectionTrait>(db: &C, job: &job_request::Model) -> AppResult<Option<Uuid>> {
    let Some(qid) = job.accepted_quote_id else { return Ok(None) };
    let q = quote::Entity::find_by_id(qid).one(db).await?;
    let Some(q) = q else { return Ok(None) };
    Ok(provider_profile::Entity::find_by_id(q.provider_id)
        .one(db)
        .await?
        .map(|p| p.user_id))
}

pub async fn provider_matches<C: ConnectionTrait>(db: &C, p: &provider_profile::Model, job: &job_request::Model) -> AppResult<bool> {
    if !p.service_postal_codes.contains(&job.postal_code) {
        return Ok(false);
    }
    Ok(provider_category::Entity::find_by_id((p.id, job.category_id))
        .one(db)
        .await?
        .is_some())
}

pub async fn access<C: ConnectionTrait>(db: &C, u: &user::Model, job_id: Uuid, lock: bool) -> AppResult<JobAccess> {
    let mut q = job_request::Entity::find_by_id(job_id);
    if lock {
        q = q.lock_exclusive();
    }
    let job = q.one(db).await?.ok_or_else(AppError::not_found)?;
    let accepted = accepted_provider_user(db, &job).await?;
    let (provider, matches) = if u.role == UserRole::Provider {
        let p = profile_for_user(db, u).await?;
        let m = provider_matches(db, &p, &job).await?;
        (Some(p), m)
    } else {
        (None, false)
    };
    let is_owner = job.customer_id == u.id;
    let has_quote = match &provider {
        Some(p) => quote::Entity::find()
            .filter(quote::Column::JobRequestId.eq(job.id))
            .filter(quote::Column::ProviderId.eq(p.id))
            .one(db)
            .await?
            .is_some(),
        None => false,
    };
    let can_view = u.role == UserRole::Admin || is_owner || accepted == Some(u.id) || has_quote || matches;
    if !can_view {
        return Err(AppError::forbidden("Do tega povpraševanja nimate dostopa."));
    }
    Ok(JobAccess {
        job,
        is_owner,
        accepted_provider_user: accepted,
        provider,
        provider_matches: matches,
    })
}

pub async fn get(db: &DatabaseConnection, u: &user::Model, job_id: Uuid) -> AppResult<JobDto> {
    let a = access(db, u, job_id, false).await?;
    let is_accepted_provider = a.is_accepted_provider(u);
    let job = a.job;
    let cat = service_category::Entity::find_by_id(job.category_id)
        .one(db)
        .await?
        .map(|c| c.name)
        .unwrap_or_default();
    let show_address = a.is_owner || is_accepted_provider;
    let customer = if is_accepted_provider {
        user::Entity::find_by_id(job.customer_id).one(db).await?.map(|c| ContactDto {
            full_name: c.full_name,
            phone: c.phone,
            business_name: None,
        })
    } else {
        None
    };
    let provider = match (a.is_owner || u.role == UserRole::Admin, a.accepted_provider_user) {
        (true, Some(pu)) => {
            let pu = user::Entity::find_by_id(pu).one(db).await?;
            let pp = provider_profile::Entity::find()
                .filter(provider_profile::Column::UserId.eq(pu.as_ref().map(|x| x.id)))
                .one(db)
                .await?;
            pu.map(|x| ContactDto {
                full_name: x.full_name,
                phone: x.phone,
                business_name: pp.map(|p| p.business_name),
            })
        }
        _ => None,
    };
    let active_quote_count = quote::Entity::find()
        .filter(quote::Column::JobRequestId.eq(job.id))
        .filter(quote::Column::Status.ne(QuoteStatus::Withdrawn))
        .count(db)
        .await? as i64;
    Ok(JobDto {
        id: job.id,
        category_id: job.category_id,
        category_name: cat,
        title: job.title,
        description: job.description,
        city: job.city,
        postal_code: job.postal_code,
        address_line: if show_address { job.address_line } else { None },
        preferred_from: job.preferred_from,
        preferred_to: job.preferred_to,
        budget_min: job.budget_min.map(crate::f),
        budget_max: job.budget_max.map(crate::f),
        status: job.status,
        accepted_quote_id: job.accepted_quote_id,
        expires_at: job.expires_at,
        created_at: job.created_at,
        active_quote_count,
        is_owner: a.is_owner,
        is_accepted_provider,
        customer,
        provider,
    })
}

pub async fn list_mine(db: &DatabaseConnection, u: &user::Model) -> AppResult<Vec<JobListItem>> {
    if u.role != UserRole::Customer {
        return Err(AppError::forbidden("To je na voljo samo strankam."));
    }
    let jobs = job_request::Entity::find()
        .filter(job_request::Column::CustomerId.eq(u.id))
        .order_by_desc(job_request::Column::CreatedAt)
        .find_also_related(service_category::Entity)
        .all(db)
        .await?;
    let mut out = Vec::with_capacity(jobs.len());
    for (j, c) in jobs {
        let quote_count = quote::Entity::find()
            .filter(quote::Column::JobRequestId.eq(j.id))
            .filter(quote::Column::Status.ne(QuoteStatus::Withdrawn))
            .count(db)
            .await? as i64;
        out.push(JobListItem {
            id: j.id,
            title: j.title,
            category_name: c.map(|c| c.name).unwrap_or_default(),
            city: j.city,
            status: j.status,
            quote_count,
            created_at: j.created_at,
        });
    }
    Ok(out)
}

const MATCHING_WHERE: &str = r#"
FROM job_requests j
JOIN service_categories c ON c.id = j.category_id
WHERE j.status IN ('OPEN', 'QUOTED')
  AND j.category_id IN (SELECT category_id FROM provider_categories WHERE provider_profile_id = $1)
  AND j.postal_code = ANY (SELECT unnest(service_postal_codes) FROM provider_profiles WHERE id = $1)
"#;

pub async fn matching(db: &DatabaseConnection, u: &user::Model, page: u64) -> AppResult<MatchingPage> {
    let p = profile_for_user(db, u).await?;
    let page = page.max(1);
    let offset = (page - 1) * PAGE_SIZE;
    let sql = format!(
        r#"SELECT j.id, j.title, c.name AS category_name, j.city, j.postal_code, j.preferred_from, j.preferred_to,
            j.budget_min::float8 AS budget_min, j.budget_max::float8 AS budget_max, j.status::text AS status,
            (SELECT COUNT(*) FROM quotes q WHERE q.job_request_id = j.id AND q.status <> 'WITHDRAWN') AS quote_count,
            EXISTS (SELECT 1 FROM quotes q WHERE q.job_request_id = j.id AND q.provider_id = $1) AS quote_sent,
            j.created_at
        {MATCHING_WHERE}
        ORDER BY j.created_at DESC
        LIMIT {PAGE_SIZE} OFFSET $2"#
    );
    let items = MatchingItem::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        sql,
        [p.id.into(), (offset as i64).into()],
    ))
    .all(db)
    .await?;
    #[derive(FromQueryResult)]
    struct Total {
        total: i64,
    }
    let total = Total::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!("SELECT COUNT(*) AS total {MATCHING_WHERE}"),
        [p.id.into()],
    ))
    .one(db)
    .await?
    .map(|t| t.total)
    .unwrap_or(0);
    let dto = crate::providers::to_dto(db, p).await?;
    Ok(MatchingPage {
        items,
        page,
        page_size: PAGE_SIZE,
        total,
        verification_status: dto.verification_status,
        category_names: dto.category_names,
        service_postal_codes: dto.service_postal_codes,
    })
}

pub async fn history(db: &DatabaseConnection, u: &user::Model, job_id: Uuid) -> AppResult<Vec<HistoryDto>> {
    access(db, u, job_id, false).await?;
    let rows = job_status_history::Entity::find()
        .filter(job_status_history::Column::JobRequestId.eq(job_id))
        .order_by_asc(job_status_history::Column::ChangedAt)
        .order_by_asc(job_status_history::Column::Id)
        .all(db)
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let by = user::Entity::find_by_id(r.changed_by).one(db).await?;
        out.push(HistoryDto {
            from_status: r.from_status,
            to_status: r.to_status,
            changed_by_name: by.as_ref().map(|b| b.full_name.clone()).unwrap_or_default(),
            changed_by_role: by.map(|b| b.role).unwrap_or(UserRole::Customer),
            note: r.note,
            changed_at: r.changed_at,
        });
    }
    Ok(out)
}

/// Status change from the job page: start, finish or cancel.
pub async fn change_status(db: &DatabaseConnection, u: &user::Model, job_id: Uuid, input: StatusInput) -> AppResult<JobDto> {
    // QUOTED and ACCEPTED have their own endpoints, EXPIRED and REVIEWED are not part of MVP
    if !matches!(input.status, JobStatus::InProgress | JobStatus::Completed | JobStatus::Cancelled) {
        return Err(AppError::invalid_transition());
    }
    let txn = db.begin().await?;
    let a = access(&txn, u, job_id, true).await?;
    let actor = a.actor(u);
    workflow::check_transition(Some(a.job.status), input.status, actor)?;
    let note = input.note.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
    if input.status == JobStatus::Cancelled && note.as_ref().is_none_or(|n| n.chars().count() < 10) {
        return Err(AppError::field("note", "Razlog mora imeti vsaj 10 znakov."));
    }
    let job = workflow::apply(&txn, a.job, input.status, actor, u.id, note, None).await?;
    if job.status == JobStatus::Cancelled {
        quote::Entity::update_many()
            .col_expr(quote::Column::Status, QuoteStatus::Rejected.as_enum())
            .filter(quote::Column::JobRequestId.eq(job.id))
            .filter(quote::Column::Status.eq(QuoteStatus::Pending))
            .exec(&txn)
            .await?;
    }
    txn.commit().await?;
    get(db, u, job_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CreateJobInput {
        CreateJobInput {
            category_id: Uuid::new_v4(),
            title: "Pušča pipa".into(),
            description: "V kuhinji pušča pipa že nekaj dni.".into(),
            city: "Maribor".into(),
            postal_code: "2000".into(),
            address_line: None,
            preferred_from: None,
            preferred_to: None,
            budget_min: None,
            budget_max: None,
        }
    }

    #[test]
    fn valid_input_passes() {
        assert!(validate(&input()).is_ok());
    }

    #[test]
    fn field_errors() {
        let mut i = input();
        i.title = "Pipa".into();
        assert_eq!(validate(&i).unwrap_err().field.as_deref(), Some("title"));
        let mut i = input();
        i.description = "prekratko".into();
        assert_eq!(validate(&i).unwrap_err().field.as_deref(), Some("description"));
        let mut i = input();
        i.postal_code = "200".into();
        assert_eq!(validate(&i).unwrap_err().field.as_deref(), Some("postal_code"));
        let mut i = input();
        i.budget_min = Some(100.0);
        i.budget_max = Some(50.0);
        assert_eq!(validate(&i).unwrap_err().field.as_deref(), Some("budget_max"));
        i.budget_max = Some(100.0);
        assert!(validate(&i).is_ok());
    }
}
