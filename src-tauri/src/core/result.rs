//! Result Type Aliases
//! 
//! Provides convenient Result type aliases for the application.

use crate::core::error::AppError;

/// Standard Result type for the application
pub type Result<T, E = AppError> = std::result::Result<T, E>;

/// Result type that always uses AppError
pub type AppResult<T> = std::result::Result<T, AppError>;
