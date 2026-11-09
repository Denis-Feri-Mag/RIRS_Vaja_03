use crate::JobStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "job_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub customer_id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub description: String,
    pub city: String,
    pub postal_code: String,
    pub address_line: Option<String>,
    pub preferred_from: Option<Date>,
    pub preferred_to: Option<Date>,
    pub budget_min: Option<Decimal>,
    pub budget_max: Option<Decimal>,
    pub status: JobStatus,
    pub accepted_quote_id: Option<Uuid>,
    pub expires_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::CustomerId", to = "super::user::Column::Id")]
    Customer,
    #[sea_orm(
        belongs_to = "super::service_category::Entity",
        from = "Column::CategoryId",
        to = "super::service_category::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::quote::Entity")]
    Quote,
    #[sea_orm(has_many = "super::job_status_history::Entity")]
    History,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::service_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::quote::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Quote.def()
    }
}

impl Related<super::job_status_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::History.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
