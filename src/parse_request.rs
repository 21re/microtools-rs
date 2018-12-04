use super::AsyncBusinessResult;
use actix_web::dev::{JsonBody, MessageBody};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use bytes::Bytes;
use futures::{Future, IntoFuture};
use problem::Problem;
use serde::de::DeserializeOwned;
use std::fmt;
use std::ops::Deref;

pub struct ValidatedJsonConfig {
  limit: usize,
}

impl Default for ValidatedJsonConfig {
  fn default() -> Self {
    ValidatedJsonConfig { limit: 1024 * 1024 }
  }
}

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
  type Config = ValidatedJsonConfig;
  type Result = Result<AsyncBusinessResult<Self>, Problem>;

  #[inline]
  fn from_request(req: &HttpRequest<S>, cfg: &Self::Config) -> Self::Result {
    Ok(Box::new(
      JsonBody::new(req)
        .limit(cfg.limit)
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

pub struct LimitedRawConfig {
  limit: usize,
}

impl Default for LimitedRawConfig {
  fn default() -> Self {
    LimitedRawConfig { limit: 1024 * 1024 }
  }
}

pub struct LimitedRaw(pub Bytes);

impl Deref for LimitedRaw {
  type Target = Bytes;

  fn deref(&self) -> &Bytes {
    &self.0
  }
}

impl<S> FromRequest<S> for LimitedRaw
where
  S: 'static,
{
  type Config = LimitedRawConfig;
  type Result = Result<AsyncBusinessResult<Self>, Problem>;

  #[inline]
  fn from_request(req: &HttpRequest<S>, cfg: &Self::Config) -> Self::Result {
    Ok(Box::new(
      MessageBody::new(req)
        .limit(cfg.limit)
        .map_err(|error| Problem::bad_request().with_details(format!("Invalid body: {}", error)))
        .map(LimitedRaw),
    ))
  }
}
