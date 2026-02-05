use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForeciseError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ForeciseError>;
