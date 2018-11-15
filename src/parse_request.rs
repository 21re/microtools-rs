use super::AsyncBusinessResult;
use actix_web::dev::JsonBody;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use futures::{Future, IntoFuture};
use problem::Problem;
use serde::de::DeserializeOwned;
use std::fmt;
use std::ops::Deref;

const JSON_REQUEST_LIMIT: usize = 1024 * 1024;

pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
  /// Deconstruct to an inner value
  pub fn into_inner(self) -> T {
    self.0
  }
}

impl<T> Deref for ValidatedJson<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.0
  }
}

impl<T> fmt::Debug for ValidatedJson<T>
where
  T: fmt::Debug,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Json: {:?}", self.0)
  }
}

impl<T> fmt::Display for ValidatedJson<T>
where
  T: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.0, f)
  }
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + 'static,
  S: 'static,
{
  type Config = ();
  type Result = Result<AsyncBusinessResult<Self>, Problem>;

  #[inline]
  fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
    Ok(Box::new(
      JsonBody::new(req)
        .limit(JSON_REQUEST_LIMIT)
        .map_err(|error| Problem::bad_request().with_details(format!("Invalid json: {}", error)))
        .map(ValidatedJson),
    ))
  }
}

pub fn validate_json<R, F, B, U, T>(http_message: &R, f: F) -> AsyncBusinessResult<T>
where
  R: HttpMessage + 'static,
  B: DeserializeOwned + 'static,
  F: FnOnce(B) -> U + 'static,
  U: IntoFuture<Item = T, Error = Problem> + 'static,
{
  Box::new(
    http_message
      .json::<B>()
      .map_err(|error| Problem::bad_request().with_details(format!("Invalid json: {}", error)))
      .and_then(f),
  )
}

#[macro_export]
macro_rules! request_parameter {
  ($req:expr, $name:expr) => {
    match $req.match_info().query($name) {
      Ok(value) => value,
      Err(error) => {
        return business_result::failure(Problem::bad_request().with_details(format!("Missing {}: {}", $name, error)))
      }
    }
  };
}
