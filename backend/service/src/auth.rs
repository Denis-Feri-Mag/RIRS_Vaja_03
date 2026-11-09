use crate::error::{AppError, AppResult};
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;
use entity::{provider_profile, user, UserRole, VerificationStatus};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterInput {
    pub email: String,
    pub full_name: String,
    pub password: String,
    pub role: UserRole,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub role: UserRole,
}

impl From<user::Model> for UserDto {
    fn from(u: user::Model) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            phone: u.phone,
            role: u.role,
        }
    }
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::new(500, "INTERNAL", "Napaka pri shranjevanju gesla."))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

fn valid_email(email: &str) -> bool {
    let mut parts = email.split('@');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), None) => !a.is_empty() && b.contains('.') && !b.starts_with('.') && !b.ends_with('.') && !email.contains(' '),
        _ => false,
    }
}

pub async fn register(db: &DatabaseConnection, input: RegisterInput) -> AppResult<UserDto> {
    let email = input.email.trim().to_lowercase();
    let full_name = input.full_name.trim().to_string();
    if !valid_email(&email) || email.len() > 255 {
        return Err(AppError::field("email", "Vpišite veljaven e-naslov."));
    }
    if full_name.is_empty() || full_name.chars().count() > 120 {
        return Err(AppError::field("full_name", "Vpišite ime in priimek."));
    }
    if input.password.chars().count() < 8 {
        return Err(AppError::field("password", "Geslo mora imeti vsaj 8 znakov."));
    }
    if input.role == UserRole::Admin {
        return Err(AppError::field("role", "Izberite vlogo Stranka ali Izvajalec."));
    }
    let phone = input.phone.map(|p| p.trim().to_string()).filter(|p| !p.is_empty());
    if phone.as_ref().is_some_and(|p| p.chars().count() > 30) {
        return Err(AppError::field("phone", "Telefon je predolg."));
    }
    let exists = user::Entity::find().filter(user::Column::Email.eq(&email)).one(db).await?;
    if exists.is_some() {
        return Err(email_taken());
    }
    let hash = hash_password(&input.password)?;
    let now = Utc::now().fixed_offset();
    let txn = db.begin().await?;
    let u = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email),
        password_hash: Set(hash),
        full_name: Set(full_name.clone()),
        phone: Set(phone),
        role: Set(input.role),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            email_taken()
        } else {
            e.into()
        }
    })?;
    if u.role == UserRole::Provider {
        new_profile(u.id, full_name, VerificationStatus::Pending).insert(&txn).await?;
    }
    txn.commit().await?;
    Ok(u.into())
}

pub(crate) fn new_profile(user_id: Uuid, name: String, status: VerificationStatus) -> provider_profile::ActiveModel {
    provider_profile::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        business_name: Set(name),
        tax_number: Set(String::new()),
        description: Set(String::new()),
        hourly_rate: Set(None),
        service_city: Set(String::new()),
        service_postal_codes: Set(vec![]),
        verification_status: Set(status),
        verified_by: Set(None),
        verified_at: Set(None),
        avg_rating: Set(Decimal::ZERO),
        completed_jobs: Set(0),
    }
}

fn email_taken() -> AppError {
    AppError::new(409, "EMAIL_TAKEN", "Račun s tem e-naslovom že obstaja.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_is_argon2id() {
        let h = hash_password("geslo1234").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(verify_password("geslo1234", &h));
        assert!(!verify_password("napacno12", &h));
    }

    #[test]
    fn email_check() {
        assert!(valid_email("ana@primer.si"));
        assert!(!valid_email("ana.primer.si"));
        assert!(!valid_email("ana@si"));
    }
}
