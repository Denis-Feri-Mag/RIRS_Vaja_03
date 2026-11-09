//! All job status rules are here, see docs/02 and ADR 0005.
use crate::error::{AppError, AppResult};
use chrono::Utc;
use entity::{job_request, job_status_history, JobStatus, UserRole};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// customer who posted the job
    OwnerCustomer,
    /// provider who sent a quote (used for the first quote)
    QuotingProvider,
    /// provider whose quote was accepted
    AcceptedProvider,
    Admin,
    System,
    Other,
}

impl Actor {
    pub fn resolve(role: UserRole, user_id: Uuid, job: &job_request::Model, accepted_provider_user: Option<Uuid>) -> Self {
        match role {
            UserRole::Admin => Actor::Admin,
            UserRole::Customer if job.customer_id == user_id => Actor::OwnerCustomer,
            UserRole::Provider if accepted_provider_user == Some(user_id) => Actor::AcceptedProvider,
            _ => Actor::Other,
        }
    }
}

/// Who may make the transition, or None if it is not in the state diagram.
fn allowed_actors(from: Option<JobStatus>, to: JobStatus) -> Option<&'static [Actor]> {
    use Actor::*;
    use JobStatus::*;
    let actors: &'static [Actor] = match (from, to) {
        (None, Open) => &[OwnerCustomer],
        (Some(Open), Quoted) => &[QuotingProvider],
        (Some(Quoted), Accepted) => &[OwnerCustomer],
        (Some(Open | Quoted), Cancelled) => &[OwnerCustomer],
        (Some(Open | Quoted), Expired) => &[System],
        (Some(Accepted), InProgress) => &[AcceptedProvider],
        (Some(Accepted), Cancelled) => &[OwnerCustomer, Admin],
        (Some(InProgress), Completed) => &[AcceptedProvider],
        (Some(InProgress), Cancelled) => &[Admin],
        (Some(Completed), Reviewed) => &[OwnerCustomer],
        _ => return None,
    };
    Some(actors)
}

pub fn check_transition(from: Option<JobStatus>, to: JobStatus, actor: Actor) -> AppResult<()> {
    match allowed_actors(from, to) {
        None => Err(AppError::invalid_transition()),
        Some(list) if list.contains(&actor) => Ok(()),
        Some(_) => Err(AppError::forbidden("Te spremembe statusa ne smete narediti.")),
    }
}

/// Checks the transition, updates the job and writes history. Call inside a transaction.
pub async fn apply<C: ConnectionTrait>(
    db: &C,
    job: job_request::Model,
    to: JobStatus,
    actor: Actor,
    changed_by: Uuid,
    note: Option<String>,
    accepted_quote_id: Option<Uuid>,
) -> AppResult<job_request::Model> {
    let from = job.status;
    check_transition(Some(from), to, actor)?;
    let now = Utc::now().fixed_offset();
    let mut am: job_request::ActiveModel = job.into();
    am.status = Set(to);
    am.updated_at = Set(now);
    if let Some(q) = accepted_quote_id {
        am.accepted_quote_id = Set(Some(q));
    }
    let updated = am.update(db).await?;
    job_status_history::ActiveModel {
        job_request_id: Set(updated.id),
        from_status: Set(Some(from)),
        to_status: Set(to),
        changed_by: Set(changed_by),
        note: Set(note),
        changed_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Actor::*;
    use JobStatus::*;

    #[test]
    fn happy_path() {
        assert!(check_transition(None, Open, OwnerCustomer).is_ok());
        assert!(check_transition(Some(Open), Quoted, QuotingProvider).is_ok());
        assert!(check_transition(Some(Quoted), Accepted, OwnerCustomer).is_ok());
        assert!(check_transition(Some(Accepted), InProgress, AcceptedProvider).is_ok());
        assert!(check_transition(Some(InProgress), Completed, AcceptedProvider).is_ok());
    }

    #[test]
    fn not_in_diagram_is_409() {
        let e = check_transition(Some(Open), Completed, AcceptedProvider).unwrap_err();
        assert_eq!(e.status, 409);
        assert_eq!(e.message, "Ta sprememba statusa ni dovoljena.");
        assert_eq!(
            check_transition(Some(Completed), InProgress, AcceptedProvider).unwrap_err().status,
            409
        );
        assert_eq!(check_transition(Some(Quoted), Quoted, QuotingProvider).unwrap_err().status, 409);
    }

    #[test]
    fn wrong_actor_is_403() {
        assert_eq!(check_transition(Some(Accepted), InProgress, OwnerCustomer).unwrap_err().status, 403);
        assert_eq!(check_transition(Some(Accepted), InProgress, Other).unwrap_err().status, 403);
        assert_eq!(check_transition(Some(InProgress), Completed, Admin).unwrap_err().status, 403);
    }
}
