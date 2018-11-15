use actix;
use actix_web;
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

impl From<actix_web::error::Error> for Problem {
  fn from(error: actix_web::Error) -> Problem {
    error!("Actix: {}", error);

    Problem::internal_server_error().with_details(format!("Actix: {}", error))
  }
}

impl actix_web::error::ResponseError for Problem {
  fn error_response(&self) -> actix_web::HttpResponse {
    actix_web::HttpResponse::build(
      actix_web::http::StatusCode::from_u16(self.code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    ).json(self)
  }
}

impl actix_web::Responder for Problem {
  type Item = actix_web::HttpResponse;
  type Error = Problem;

  fn respond_to<S: 'static>(self, _req: &actix_web::HttpRequest<S>) -> Result<actix_web::HttpResponse, Problem> {
    Ok(
      actix_web::HttpResponse::build(
        actix_web::http::StatusCode::from_u16(self.code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
      ).json(self),
    )
  }
}

impl From<actix_web::client::SendRequestError> for Problem {
  fn from(error: actix_web::client::SendRequestError) -> Problem {
    use actix_web::client::SendRequestError::*;

    error!("Http client: {}", error);

    match error {
      Timeout => Problem::internal_server_error().with_details("Request timeout"),
      Connector(err) => Problem::internal_server_error().with_details(format!("HTTP connection error: {}", err)),
      ParseError(err) => Problem::internal_server_error().with_details(format!("Invalid HTTP response: {}", err)),
      Io(err) => Problem::from(err),
    }
  }
}

impl From<actix_web::error::PayloadError> for Problem {
  fn from(error: actix_web::error::PayloadError) -> Self {
    error!("Http payload: {}", error);
    Problem::internal_server_error().with_details(format!("Http payload: {}", error))
  }
}

impl From<actix_web::error::ReadlinesError> for Problem {
  fn from(error: actix_web::error::ReadlinesError) -> Self {
    use actix_web::error::ReadlinesError::*;

    match error {
      EncodingError => Problem::internal_server_error().with_details("Readline: Invalid encoding"),
      PayloadError(error) => Problem::from(error),
      LimitOverflow => Problem::internal_server_error().with_details("Readline: Limit exeeded"),
      ContentTypeError(error) => Problem::from(error),
    }
  }
}

impl From<actix_web::error::ContentTypeError> for Problem {
  fn from(error: actix_web::error::ContentTypeError) -> Self {
    error!("Http content type: {}", error);

    Problem::internal_server_error().with_details(format!("Http content type: {}", error))
  }
}

impl From<actix_web::error::JsonPayloadError> for Problem {
  fn from(error: actix_web::error::JsonPayloadError) -> Self {
    error!("Http json type: {}", error);

    Problem::internal_server_error().with_details(format!("Http json payload: {}", error))
  }
}

impl From<actix::MailboxError> for Problem {
  fn from(error: actix::MailboxError) -> Self {
    error!("Actix mailbox error: {}", error);

    Problem::internal_server_error().with_details(format!("Actix mailbox error: {}", error))
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
