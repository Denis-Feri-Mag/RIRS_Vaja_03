use crate::error::{AppError, AppResult};
use entity::{provider_category, provider_profile, service_category, user, UserRole, VerificationStatus};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct ProviderProfileDto {
    pub id: Uuid,
    pub business_name: String,
    pub tax_number: String,
    pub description: String,
    pub service_city: String,
    pub service_postal_codes: Vec<String>,
    pub category_ids: Vec<Uuid>,
    pub category_names: Vec<String>,
    pub verification_status: VerificationStatus,
    pub avg_rating: f64,
    pub completed_jobs: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProviderProfileInput {
    pub business_name: String,
    pub tax_number: String,
    pub description: String,
    pub service_city: String,
    pub service_postal_codes: Vec<String>,
    pub category_ids: Vec<Uuid>,
}

pub fn valid_postal_code(p: &str) -> bool {
    p.len() == 4 && p.chars().all(|c| c.is_ascii_digit())
}

pub async fn profile_for_user<C: ConnectionTrait>(db: &C, u: &user::Model) -> AppResult<provider_profile::Model> {
    if u.role != UserRole::Provider {
        return Err(AppError::forbidden("To je na voljo samo izvajalcem."));
    }
    provider_profile::Entity::find()
        .filter(provider_profile::Column::UserId.eq(u.id))
        .one(db)
        .await?
        .ok_or_else(AppError::not_found)
}

pub async fn to_dto<C: ConnectionTrait>(db: &C, p: provider_profile::Model) -> AppResult<ProviderProfileDto> {
    let cats = p.find_related(service_category::Entity).all(db).await?;
    Ok(ProviderProfileDto {
        id: p.id,
        business_name: p.business_name,
        tax_number: p.tax_number,
        description: p.description,
        service_city: p.service_city,
        service_postal_codes: p.service_postal_codes,
        category_ids: cats.iter().map(|c| c.id).collect(),
        category_names: cats.into_iter().map(|c| c.name).collect(),
        verification_status: p.verification_status,
        avg_rating: crate::f(p.avg_rating),
        completed_jobs: p.completed_jobs,
    })
}

pub async fn get_mine(db: &DatabaseConnection, u: &user::Model) -> AppResult<ProviderProfileDto> {
    let p = profile_for_user(db, u).await?;
    to_dto(db, p).await
}

/// Provider edits his own profile. Verification status is not changed here.
pub async fn update_mine(db: &DatabaseConnection, u: &user::Model, input: ProviderProfileInput) -> AppResult<ProviderProfileDto> {
    let p = profile_for_user(db, u).await?;
    let name = input.business_name.trim().to_string();
    if name.is_empty() || name.chars().count() > 150 {
        return Err(AppError::field("business_name", "Vpišite naziv."));
    }
    if input.tax_number.trim().chars().count() > 20 {
        return Err(AppError::field("tax_number", "Davčna številka je predolga."));
    }
    if input.service_city.trim().chars().count() > 100 {
        return Err(AppError::field("service_city", "Kraj je predolg."));
    }
    let mut codes: Vec<String> = input
        .service_postal_codes
        .iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    if codes.iter().any(|c| !valid_postal_code(c)) {
        return Err(AppError::field("service_postal_codes", "Poštna številka mora imeti 4 števke."));
    }
    codes.sort();
    codes.dedup();
    let mut cat_ids = input.category_ids.clone();
    cat_ids.sort();
    cat_ids.dedup();
    let active = service_category::Entity::find()
        .filter(service_category::Column::Id.is_in(cat_ids.clone()))
        .filter(service_category::Column::IsActive.eq(true))
        .all(db)
        .await?;
    if active.len() != cat_ids.len() {
        return Err(AppError::field("category_ids", "Izberite aktivne kategorije."));
    }
    let txn = db.begin().await?;
    let mut am: provider_profile::ActiveModel = p.into();
    am.business_name = Set(name);
    am.tax_number = Set(input.tax_number.trim().to_string());
    am.description = Set(input.description.trim().to_string());
    am.service_city = Set(input.service_city.trim().to_string());
    am.service_postal_codes = Set(codes);
    let p = am.update(&txn).await?;
    provider_category::Entity::delete_many()
        .filter(provider_category::Column::ProviderProfileId.eq(p.id))
        .exec(&txn)
        .await?;
    for c in cat_ids {
        provider_category::ActiveModel {
            provider_profile_id: Set(p.id),
            category_id: Set(c),
        }
        .insert(&txn)
        .await?;
    }
    let dto = to_dto(&txn, p).await?;
    txn.commit().await?;
    Ok(dto)
}
