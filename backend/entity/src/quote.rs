use crate::QuoteStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "quotes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub job_request_id: Uuid,
    pub provider_id: Uuid,
    pub price: Decimal,
    pub estimated_hours: Decimal,
    pub earliest_start: Date,
    pub message: Option<String>,
    pub status: QuoteStatus,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::job_request::Entity",
        from = "Column::JobRequestId",
        to = "super::job_request::Column::Id"
    )]
    JobRequest,
    #[sea_orm(
        belongs_to = "super::provider_profile::Entity",
        from = "Column::ProviderId",
        to = "super::provider_profile::Column::Id"
    )]
    Provider,
}

impl Related<super::job_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobRequest.def()
    }
}

impl Related<super::provider_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provider.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
