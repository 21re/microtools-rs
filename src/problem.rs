use actix_web::HttpResponseBuilder;
use awc::error::SendRequestError;
use log::error;
use serde_derive::{Deserialize, Serialize};

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

impl actix_web::ResponseError for Problem {
  fn status_code(&self) -> actix_web::http::StatusCode {
    actix_web::http::StatusCode::from_u16(self.code).unwrap()
  }

  fn error_response(&self) -> actix_web::HttpResponse {
    HttpResponseBuilder::new(self.status_code()).json(&self)
  }
}

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

impl From<actix::MailboxError> for Problem {
  fn from(error: actix::MailboxError) -> Self {
    error!("Actix mailbox error: {}", error);

    Problem::internal_server_error().with_details(format!("Actix mailbox error: {}", error))
  }
}

impl From<std::time::SystemTimeError> for Problem {
  fn from(error: std::time::SystemTimeError) -> Problem {
    error!("SystemTime error: {}", error);

    Problem::internal_server_error().with_details(format!("SystemTime error: {}", error))
  }
}

impl From<SendRequestError> for Problem {
  fn from(error: SendRequestError) -> Problem {
    use SendRequestError::*;

    error!("Http client: {}", error);

    match error {
      Timeout => Problem::internal_server_error().with_details("Request timeout"),
      Connect(err) => Problem::internal_server_error().with_details(format!("HTTP connection error: {}", err)),
      Response(err) => Problem::internal_server_error().with_details(format!("Invalid HTTP response: {}", err)),
      Send(err) => Problem::from(err),
      _ => Problem::internal_server_error().with_details(format!("HTTP client error: {}", error)),
    }
  }
}

impl From<actix_web::error::PayloadError> for Problem {
  fn from(error: actix_web::error::PayloadError) -> Self {
    error!("Http payload: {}", error);
    Problem::internal_server_error().with_details(format!("Http payload: {}", error))
  }
}

impl From<awc::error::JsonPayloadError> for Problem {
  fn from(error: awc::error::JsonPayloadError) -> Self {
    error!("Http json type: {}", error);

    Problem::internal_server_error().with_details(format!("Http json payload: {}", error))
  }
}

impl From<serde_json::Error> for Problem {
  fn from(error: serde_json::Error) -> Self {
    error!("Json: {}", error);

    Problem::internal_server_error().with_details(format!("Json: {}", error))
  }
}

impl From<::reqwest::Error> for Problem {
  fn from(error: ::reqwest::Error) -> Self {
    error!("Reqwest error: {}", error);

    Problem::internal_server_error().with_details(format!("Request result: {}", error))
  }
}
