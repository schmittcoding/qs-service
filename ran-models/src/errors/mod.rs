use std::collections::HashMap;

use base64::Engine;
use deadpool_tiberius::deadpool::managed::PoolError;
use tiberius::error::Error;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(PoolError<Error>),
    QueryError(Error),
    UserNotFound,
    AuthenticationFailed,
    TestConnectionFailed,
    DatabaseNotConfigured,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            DatabaseError::QueryError(e) => write!(f, "Query error: {}", e),
            DatabaseError::UserNotFound => write!(f, "User not found"),
            DatabaseError::AuthenticationFailed => write!(f, "Authentication failed"),
            DatabaseError::TestConnectionFailed => write!(f, "Test connection failed"),
            DatabaseError::DatabaseNotConfigured => write!(f, "Database not configured"),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<PoolError<Error>> for DatabaseError {
    fn from(err: PoolError<Error>) -> Self {
        DatabaseError::ConnectionError(err)
    }
}

impl From<tiberius::error::Error> for DatabaseError {
    fn from(err: tiberius::error::Error) -> Self {
        DatabaseError::QueryError(err)
    }
}

pub fn flatten_validation_errors(errors: &ValidationErrors) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (field, field_errors) in errors.field_errors() {
        if let Some(error) = field_errors.get(0) {
            let message = error.message.clone().unwrap_or_else(|| error.code.clone());
            map.insert(field.to_string(), message.to_string());
        }
    }
    map
}

pub fn validate_base64_optional(value: &str) -> Result<(), ValidationError> {
    base64::engine::general_purpose::STANDARD
        .decode(value.trim())
        .map_err(|_| ValidationError::new("invalid_base64"))?;
    Ok(())
}
