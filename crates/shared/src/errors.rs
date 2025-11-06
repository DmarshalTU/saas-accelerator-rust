use thiserror::Error;

/// Common error types for the SaaS Accelerator
#[derive(Error, Debug)]
pub enum AcceleratorError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Marketplace API error: {0}")]
    Marketplace(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AcceleratorError>;

