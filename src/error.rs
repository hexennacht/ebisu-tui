use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] libsql::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Insufficient funds in category and overflow chain")]
    InsufficientFunds,

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
