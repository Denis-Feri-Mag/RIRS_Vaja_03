use crate::error::AppResult;
use entity::service_category;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

pub async fn list_active(db: &DatabaseConnection) -> AppResult<Vec<CategoryDto>> {
    Ok(service_category::Entity::find()
        .filter(service_category::Column::IsActive.eq(true))
        .order_by_asc(service_category::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id,
            name: c.name,
            slug: c.slug,
        })
        .collect())
}
