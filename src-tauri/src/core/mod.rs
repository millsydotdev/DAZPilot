//! Core module - foundational types and utilities
//!
//! This module contains the core error types and result aliases
//! that are used throughout the application.

pub mod error;
pub mod result;

pub use error::AppError;
pub use result::Result;
