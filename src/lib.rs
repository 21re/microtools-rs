#![crate_type = "lib"]

pub mod auth_middleware;
pub mod business_result;
pub mod elasticsearch;
#[cfg(test)]
pub mod elasticsearch_test;
#[cfg(feature = "with-slog")]
pub mod logging_slog;
pub mod metrics;
mod problem;
pub mod serde_field_value;
mod service_requester;
#[cfg(test)]
mod service_requester_test;
pub mod status;
pub mod subject;
pub mod types;
pub mod ws_try;

pub use crate::business_result::{AsyncBusinessResult, BusinessResult, BusinessResultExt};
pub use crate::problem::*;
pub use crate::service_requester::*;
