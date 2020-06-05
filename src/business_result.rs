use crate::problem::Problem;
use futures::{future, Future, FutureExt};
use std::convert::Into;
use std::fmt::Display;
use std::pin::Pin;
use std::result::Result;

pub type BusinessResult<T> = Result<T, Problem>;

pub trait BusinessResultExt<T> {
  fn chain_problem<D: Display>(self, details: D) -> BusinessResult<T>;
}

impl<T, E> BusinessResultExt<T> for Result<T, E> {
  fn chain_problem<D: Display>(self, details: D) -> BusinessResult<T> {
    match self {
      Ok(result) => Ok(result),
      Err(_) => Err(Problem::internal_server_error().with_details(details)),
    }
  }
}

pub type AsyncBusinessResult<T> = Pin<Box<dyn Future<Output = BusinessResult<T>>>>;

pub fn success<T: 'static>(result: T) -> AsyncBusinessResult<T> {
  Box::pin(future::ok(result))
}

pub fn failure<T: 'static, E: Into<Problem>>(error: E) -> AsyncBusinessResult<T> {
  let problem = error.into();

  Box::pin(future::err(problem))
}

pub fn from_future<F, E, T>(f: F) -> AsyncBusinessResult<T>
where
  F: Future<Output = Result<T, E>> + 'static,
  E: Into<Problem>,
{
  Box::pin(f.map(|r| r.map_err(E::into)))
}
