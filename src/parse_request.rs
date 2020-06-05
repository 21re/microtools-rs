use crate::problem::Problem;
use bytes::Bytes;
use futures::Future;
use serde::de::DeserializeOwned;
use std::fmt;
use std::ops::Deref;

pub struct ValidatedJsonConfig {
  pub limit: usize,
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

#[macro_export]
macro_rules! request_parameter {
  ($req:expr, $name:expr) => {
    match $req.match_info().query($name) {
      Ok(value) => value,
      Err(error) => {
        return business_result::failure(Problem::bad_request().with_details(format!("Missing {}: {}", $name, error)));
      }
    }
  };
}

pub struct LimitedRawConfig {
  pub limit: usize,
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
