use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider_profile_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub category_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provider_profile::Entity",
        from = "Column::ProviderProfileId",
        to = "super::provider_profile::Column::Id"
    )]
    Provider,
    #[sea_orm(
        belongs_to = "super::service_category::Entity",
        from = "Column::CategoryId",
        to = "super::service_category::Column::Id"
    )]
    Category,
}

impl ActiveModelBehavior for ActiveModel {}
