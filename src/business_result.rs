use crate::problem::Problem;
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
