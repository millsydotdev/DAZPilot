//! Application Error Types
//! 
//! Defines the core error types used throughout the application.

use thiserror::Error;

/// Main application error enum
/// All errors in the application should map to one of these variants
#[derive(Debug, Error)]
pub enum AppError {
    // ==================== DATABASE ERRORS ====================
    
    /// Database connection or query failed
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    // ==================== NETWORK ERRORS ====================
    
    /// Network connection failed
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    // ==================== NOT FOUND ERRORS ====================
    
    /// Requested resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    // ==================== INPUT ERRORS ====================
    
    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    // ==================== PERMISSION ERRORS ====================
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    // ==================== OLLAMA ERRORS ====================
    
    /// Ollama AI service error
    #[error("AI error: {0}")]
    Ollama(#[from] OllamaError),
    
    // ==================== DAZ3D ERRORS ====================
    
    /// Daz3D plugin/connection error
    #[error("Daz3D error: {0}")]
    Daz3D(String),
    
    // ==================== PARSING ERRORS ====================
    
    /// Failed to parse or serialize data
    #[error("Parse error: {0}")]
    Parse(String),
    
    // ==================== RUNTIME ERRORS ====================
    
    /// Internal runtime error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Lock error (failed to acquire lock)
    #[error("Lock error: {0}")]
    LockError(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Database-specific errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Failed to connect to database
    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(String),
    
    /// Query execution failed
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    /// Migration failed
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),
}

/// Network-specific errors
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Connection refused or failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// Request timeout
    #[error("Request timed out: {0}")]
    Timeout(String),
    
    /// Invalid response received
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Service unavailable
    #[error("Service unavailable: {0}")]
    Unavailable(String),
}

/// Ollama AI service errors
#[derive(Debug, Error)]
pub enum OllamaError {
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Model not loaded
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    
    /// Inference failed
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    
    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),
    
    /// Ollama service not running
    #[error("Ollama not running: {0}")]
    NotRunning(String),
    
    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Network(NetworkError::ConnectionFailed(err.to_string()))
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(DatabaseError::QueryFailed(err.to_string()))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(NetworkError::ConnectionFailed(err.to_string()))
    }
}
