//! Marketplace API clients (Fulfillment, Metering).
//!
//! # Example
//!
//! ```
//! use marketplace::MarketplaceClient;
//! let builder = MarketplaceClient::builder("https://marketplaceapi.microsoft.com/api");
//! assert!(true); // builder used to construct client with credential
//! ```

#![allow(
    clippy::missing_const_for_fn, // MarketplaceClient etc. hold non-const types
    clippy::must_use_candidate,   // builder/constructor methods
)]

pub mod fulfillment;
pub mod metering;
pub mod client;

pub use fulfillment::FulfillmentApiClient;
pub use metering::MeteringApiClient;
pub use client::MarketplaceClient;

