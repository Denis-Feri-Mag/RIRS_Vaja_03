use crate::VerificationStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub business_name: String,
    pub tax_number: String,
    pub description: String,
    pub hourly_rate: Option<Decimal>,
    pub service_city: String,
    pub service_postal_codes: Vec<String>,
    pub verification_status: VerificationStatus,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTimeWithTimeZone>,
    pub avg_rating: Decimal,
    pub completed_jobs: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
    #[sea_orm(has_many = "super::quote::Entity")]
    Quote,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::quote::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quote.def()
    }
}

impl Related<super::service_category::Entity> for Entity {
    fn to() -> RelationDef {
        super::provider_category::Relation::Category.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::provider_category::Relation::Provider.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
