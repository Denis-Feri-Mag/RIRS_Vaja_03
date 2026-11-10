//! Startup data: first admin from .env and optional demo users for local testing.
use crate::auth::{hash_password, new_profile};
use crate::error::AppResult;
use chrono::Utc;
use entity::{provider_category, service_category, user, UserRole, VerificationStatus};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

async fn ensure_user(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
    name: &str,
    phone: Option<&str>,
    role: UserRole,
) -> AppResult<Option<user::Model>> {
    let email = email.trim().to_lowercase();
    if user::Entity::find().filter(user::Column::Email.eq(&email)).one(db).await?.is_some() {
        return Ok(None);
    }
    let now = Utc::now().fixed_offset();
    let u = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email),
        password_hash: Set(hash_password(password)?),
        full_name: Set(name.to_string()),
        phone: Set(phone.map(str::to_string)),
        role: Set(role),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(Some(u))
}

pub async fn ensure_admin(db: &DatabaseConnection, email: &str, password: &str) -> AppResult<()> {
    if ensure_user(db, email, password, "Administrator", None, UserRole::Admin)
        .await?
        .is_some()
    {
        tracing::info!("ustvarjen prvi administrator");
    }
    Ok(())
}

/// Demo customer and a verified provider for Maribor, so the whole flow can be tried before F8.
pub async fn seed_demo(db: &DatabaseConnection, password: &str) -> AppResult<()> {
    ensure_user(
        db,
        "stranka@popravi.test",
        password,
        "Ana Novak",
        Some("040 111 222"),
        UserRole::Customer,
    )
    .await?;
    for (email, name, phone, status) in [
        ("mojster@popravi.test", "Marko Kovač", "041 333 444", VerificationStatus::Verified),
        (
            "novi.mojster@popravi.test",
            "Jure Horvat",
            "031 555 666",
            VerificationStatus::Pending,
        ),
    ] {
        let Some(u) = ensure_user(db, email, password, name, Some(phone), UserRole::Provider).await? else {
            continue;
        };
        let mut p = new_profile(u.id, format!("{name} s.p."), status);
        p.tax_number = Set("12345678".into());
        p.service_city = Set("Maribor".into());
        p.service_postal_codes = Set(vec!["2000".into(), "2341".into()]);
        if status == VerificationStatus::Verified {
            p.verified_at = Set(Some(Utc::now().fixed_offset()));
        }
        let p = p.insert(db).await?;
        for slug in ["vodovodna-dela", "elektrika"] {
            if let Some(c) = service_category::Entity::find()
                .filter(service_category::Column::Slug.eq(slug))
                .one(db)
                .await?
            {
                provider_category::ActiveModel {
                    provider_profile_id: Set(p.id),
                    category_id: Set(c.id),
                }
                .insert(db)
                .await?;
            }
        }
    }
    tracing::info!("demo podatki so pripravljeni");
    Ok(())
}
