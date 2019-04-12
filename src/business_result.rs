use crate::problem::Problem;
use futures::{future, Future, IntoFuture};
use std::convert::Into;
use std::fmt::Display;
use std::result::Result;

pub type BusinessResult<T> = Result<T, Problem>;

pub trait BusinessResultExt<T> {
  fn chain_problem<D: Display>(self, details: D) -> BusinessResult<T>;
}

impl<T, E> BusinessResultExt<T> for Result<T, E>
where
  E: Display,
{
  fn chain_problem<D: Display>(self, details: D) -> BusinessResult<T> {
    match self {
      Ok(result) => Ok(result),
      Err(e) => Err(Problem::internal_server_error().with_details(format!("{}: {}", details, e))),
    }
  }
}

pub type AsyncBusinessResult<T> = Box<Future<Item = T, Error = Problem>>;

pub fn success<T: 'static>(result: T) -> AsyncBusinessResult<T> {
  Box::new(future::ok(result))
}

pub fn failure<T: 'static, E: Into<Problem>>(error: E) -> AsyncBusinessResult<T> {
  let problem = error.into();

  Box::new(future::err(problem))
}

pub fn from_future<F, E, T>(f: F) -> AsyncBusinessResult<T>
where
  F: IntoFuture<Item = T, Error = E> + 'static,
  E: Into<Problem> + 'static,
{
  Box::new(f.into_future().map_err(E::into))
}
