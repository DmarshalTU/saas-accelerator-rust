//! Shared models, config, auth, and services.
//!
//! # Example
//!
//! ```
//! use shared::models::SubscriptionStatus;
//! let status = SubscriptionStatus::Subscribed;
//! assert_eq!(serde_json::to_string(&status).unwrap(), r#""Subscribed""#);
//! ```

pub mod models;
pub mod config;
pub mod constants;
pub mod errors;
pub mod auth;
pub mod services;
pub mod utilities;
pub mod secrets;
