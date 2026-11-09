use sea_orm::DbErr;

#[derive(Debug, Clone, PartialEq)]
pub struct AppError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    pub field: Option<String>,
}

impl AppError {
    pub fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            field: None,
        }
    }
    pub fn field(field: &str, message: impl Into<String>) -> Self {
        Self {
            status: 400,
            code: "VALIDATION_ERROR",
            message: message.into(),
            field: Some(field.to_string()),
        }
    }
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(401, "UNAUTHORIZED", message)
    }
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403, "FORBIDDEN", message)
    }
    pub fn not_found() -> Self {
        Self::new(404, "NOT_FOUND", "Ne obstaja.")
    }
    pub fn invalid_transition() -> Self {
        Self::new(409, "INVALID_TRANSITION", "Ta sprememba statusa ni dovoljena.")
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        tracing::error!("db error: {e}");
        Self::new(500, "INTERNAL", "Prišlo je do napake na strežniku.")
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.status, self.code, self.message)
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;
