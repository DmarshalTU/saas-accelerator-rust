//! Data layer: `PostgreSQL` models and repositories.
#![allow(
    clippy::missing_const_for_fn, // DbPool (SharedPool) is not valid in const context
    clippy::needless_raw_string_hashes, // SQL strings often use r#" for readability
    clippy::must_use_candidate, // repository constructors
)]
//!
//! # Example
//!
//! ```
//! use data::models::User;
//! let user = User { user_id: 1, email_address: Some("a@b.com".into()), created_date: None, full_name: None };
//! assert_eq!(user.user_id, 1);
//! ```

pub mod models;
pub mod repositories;
pub mod pool;

pub use pool::DbPool;

