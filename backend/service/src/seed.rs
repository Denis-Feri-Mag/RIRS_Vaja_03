//! Startup data: first admin from .env.
use crate::auth::hash_password;
use crate::error::AppResult;
use chrono::Utc;
use entity::{user, UserRole};
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
