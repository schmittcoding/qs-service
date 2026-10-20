use deadpool_tiberius::deadpool::managed::BuildError;
use deadpool_tiberius::deadpool::managed::PoolError;
use deadpool_tiberius::Manager;
use std::fmt;
use thiserror::Error;

/// Specialized error types for database operations
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Duplicate record error: {0}")]
    DuplicateError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Environment configuration error: {0}")]
    ConfigError(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Unknown database error: {0}")]
    UnknownError(String),
}

impl From<tiberius::error::Error> for DbError {
    fn from(err: tiberius::error::Error) -> Self {
        // Simplified error handling based on error message content
        let err_string = err.to_string();

        if err_string.contains("connection") || err_string.contains("socket") {
            DbError::ConnectionError(err_string)
        } else if err_string.contains("tls") || err_string.contains("encryption") {
            DbError::SecurityError(err_string)
        } else {
            DbError::QueryError(err_string)
        }
    }
}

impl From<BuildError> for DbError {
    fn from(err: BuildError) -> Self {
        DbError::ConnectionError(format!("Failed to create connection pool: {}", err))
    }
}

impl From<PoolError<Manager>> for DbError {
    fn from(_err: PoolError<Manager>) -> Self {
        DbError::ConnectionError("Failed to get connection from pool".to_string())
    }
}

impl From<validator::ValidationErrors> for DbError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let error_string = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|error| error.message.clone().unwrap_or_default().to_string())
                    .collect();
                format!("{}: {}", field, error_messages.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");

        DbError::ValidationError(error_string)
    }
}

/// Security-related error information
#[derive(Debug)]
pub struct SecurityErrorInfo {
    pub severity: SecuritySeverity,
    pub message: String,
    pub source_ip: Option<String>,
    pub user_id: Option<i64>,
}

#[derive(Debug)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Helper function to log security incidents
pub fn log_security_incident(info: SecurityErrorInfo) {
    tracing::error!(
        severity = %info.severity,
        user_id = info.user_id,
        source_ip = info.source_ip.unwrap_or_default(),
        "SECURITY INCIDENT: {}",
        info.message
    );

    // You could also add functionality to insert these incidents into a security log table
}

/// Convert anyhow errors to DbError for better context
pub fn convert_anyhow_error(err: anyhow::Error) -> DbError {
    let err_string = err.to_string();

    if err_string.contains("connection") || err_string.contains("timeout") {
        DbError::ConnectionError(err_string)
    } else if err_string.contains("duplicate") || err_string.contains("already exists") {
        DbError::DuplicateError(err_string)
    } else if err_string.contains("not found") || err_string.contains("does not exist") {
        DbError::NotFoundError(err_string)
    } else if err_string.contains("transaction") {
        DbError::TransactionError(err_string)
    } else if err_string.contains("authentication") || err_string.contains("permission") {
        DbError::AuthError(err_string)
    } else if err_string.contains("validation") {
        DbError::ValidationError(err_string)
    } else if err_string.contains("config") || err_string.contains("environment") {
        DbError::ConfigError(err_string)
    } else {
        DbError::UnknownError(err_string)
    }
}
