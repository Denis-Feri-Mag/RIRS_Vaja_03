use crate::JobStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "job_status_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_request_id: Uuid,
    pub from_status: Option<JobStatus>,
    pub to_status: JobStatus,
    pub changed_by: Uuid,
    pub note: Option<String>,
    pub changed_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::job_request::Entity",
        from = "Column::JobRequestId",
        to = "super::job_request::Column::Id"
    )]
    JobRequest,
}

impl Related<super::job_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JobRequest.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
