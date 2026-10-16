//! # Web Server Module
//!
//! Exposes the Axum HTTP router and server runner.

pub mod routes;

pub use routes::build_router;
