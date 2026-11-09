use crate::UserRole;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::provider_profile::Entity")]
    ProviderProfile,
    #[sea_orm(has_many = "super::job_request::Entity")]
    JobRequest,
}

impl Related<super::provider_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProviderProfile.def()
    }
}

impl Related<super::job_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
