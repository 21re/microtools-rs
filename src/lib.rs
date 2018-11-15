#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[cfg(feature = "with-diesel")]
extern crate diesel;
extern crate prometheus;
#[cfg(feature = "with-diesel")]
extern crate r2d2;
#[cfg(feature = "with-slog")]
extern crate slog;
#[cfg(feature = "with-slog")]
extern crate slog_async;
#[cfg(feature = "with-slog")]
extern crate slog_envlogger;
#[cfg(feature = "with-slog")]
extern crate slog_json;
#[cfg(feature = "with-slog")]
extern crate slog_scope;
#[cfg(feature = "with-slog")]
extern crate slog_stdlog;
#[cfg(feature = "with-toml")]
extern crate toml;

#[cfg(test)]
extern crate spectral;

pub mod auth_middleware;
pub mod business_result;
pub mod elasticsearch;
#[cfg(test)]
pub mod elasticsearch_test;
pub mod gatekeeper;
#[cfg(feature = "with-slog")]
pub mod logging_slog;
pub mod metrics;
pub mod parse_request;
mod problem;
pub mod retry;
#[cfg(test)]
mod retry_tests;
pub mod serde_field_value;
pub mod service_requester;
pub mod status;
pub mod subject;
pub mod types;
pub mod ws_try;

pub use business_result::{AsyncBusinessResult, BusinessResult, BusinessResultExt};
pub use parse_request::*;
pub use problem::*;
pub use service_requester::*;
