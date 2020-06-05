use actix;
use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Problem {
  pub code: u16,
  #[serde(rename = "type")]
  pub problem_type: String,
  pub reason: String,
  pub details: Option<String>,
}

impl Problem {
  pub fn for_status<S: Into<String>>(code: u16, reason: S) -> Problem {
    Problem {
      code,
      problem_type: format!("https://httpstatus.es/{}", code),
      reason: reason.into(),
      details: None,
    }
  }

  pub fn bad_request() -> Problem {
    Self::for_status(400, "Bad request")
  }

  pub fn unauthorized() -> Problem {
    Self::for_status(401, "Unauthorized")
  }

  pub fn forbidden() -> Problem {
    Self::for_status(403, "Forbidden")
  }

  pub fn conflict() -> Problem {
    Self::for_status(409, "Conflict")
  }

  pub fn internal_server_error() -> Problem {
    Self::for_status(500, "Internal server error")
  }

  pub fn not_found() -> Problem {
    Self::for_status(404, "Not found")
  }

  pub fn method_not_allowed() -> Problem {
    Self::for_status(405, "Method not allowed")
  }

  pub fn failed_dependency() -> Problem {
    Self::for_status(424, "Failed dependency")
  }

  pub fn with_details<T: std::fmt::Display>(mut self, details: T) -> Problem {
    self.details = match self.details {
      Some(existing) => Some(format!("{}: {}", existing, details)),
      None => Some(format!("{}", details)),
    };
    self
  }
}

impl std::fmt::Display for Problem {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    match self.details {
      Some(ref details) => write!(
        f,
        "Problem(code={}, reason={}, details={})",
        self.code, self.reason, details
      )?,
      None => write!(f, "Problem(code={}, reason={})", self.code, self.reason)?,
    };
    Ok(())
  }
}

impl std::error::Error for Problem {}

impl From<std::env::VarError> for Problem {
  fn from(error: std::env::VarError) -> Problem {
    use std::env::VarError::*;

    match error {
      NotPresent => Problem::internal_server_error().with_details("Environment variable missing"),
      NotUnicode(_) => Problem::internal_server_error().with_details("Environment variable not unicode"),
    }
  }
}

impl From<std::io::Error> for Problem {
  fn from(error: std::io::Error) -> Problem {
    error!("IO: {}", error);

    Problem::internal_server_error().with_details(format!("IO: {}", error))
  }
}

impl<T> From<std::sync::PoisonError<T>> for Problem {
  fn from(error: std::sync::PoisonError<T>) -> Problem {
    error!("Sync poison: {}", error);

    Problem::internal_server_error().with_details(format!("Sync poison: {}", error))
  }
}

impl From<std::time::SystemTimeError> for Problem {
  fn from(error: std::time::SystemTimeError) -> Problem {
    error!("SystemTime error: {}", error);

    Problem::internal_server_error().with_details(format!("SystemTime error: {}", error))
  }
}

impl From<std::str::Utf8Error> for Problem {
  fn from(error: std::str::Utf8Error) -> Problem {
    error!("UTF-8 error: {}", error);

    Problem::bad_request().with_details(format!("UTF-8 error: {}", error))
  }
}

impl From<actix::MailboxError> for Problem {
  fn from(error: actix::MailboxError) -> Problem {
    error!("Actix error: {}", error);

    Problem::internal_server_error().with_details(format!("Actix error: {}", error))
  }
}

impl From<reqwest::Error> for Problem {
  fn from(error: reqwest::Error) -> Problem {
    error!("Reqwest: {}", error);

    Problem::internal_server_error().with_details(format!("Reqwest: {}", error))
  }
}

#[cfg(feature = "with-toml")]
impl From<::toml::de::Error> for Problem {
  fn from(error: ::toml::de::Error) -> Self {
    error!("Toml: {}", error);

    Problem::internal_server_error().with_details(format!("Toml: {}", error))
  }
}

impl From<serde_json::Error> for Problem {
  fn from(error: serde_json::Error) -> Self {
    error!("Json: {}", error);

    Problem::internal_server_error().with_details(format!("Json: {}", error))
  }
}

#[cfg(feature = "with-diesel")]
impl From<::r2d2::Error> for Problem {
  fn from(error: ::r2d2::Error) -> Self {
    error!("R2D2: {}", error);

    Problem::internal_server_error().with_details(format!("R2D2: {}", error))
  }
}

#[cfg(feature = "with-diesel")]
impl From<::diesel::result::Error> for Problem {
  fn from(error: ::diesel::result::Error) -> Self {
    error!("Diesel result: {}", error);

    Problem::internal_server_error().with_details(format!("Diesel result: {}", error))
  }
}

#[cfg(feature = "with-reqwest")]
impl From<::reqwest::Error> for Problem {
  fn from(error: ::reqwest::Error) -> Self {
    error!("Reqwest error: {}", error);

    Problem::internal_server_error().with_details(format!("Request result: {}", error))
  }
}

#[cfg(feature = "with-config")]
impl From<::config::ConfigError> for Problem {
  fn from(error: ::config::ConfigError) -> Self {
    error!("Config error: {}", error);

    Problem::internal_server_error().with_details(format!("Config: {}", error))
  }
}
