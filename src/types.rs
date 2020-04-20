use crate::problem::Problem;
use actix_web::{HttpRequest, HttpResponse, Responder};

use futures::future::{ready, Ready};

#[derive(Clone, Debug)]
pub struct Done;

impl Responder for Done {

  type Error = Problem;
  type Future = Ready<Result<actix_web::HttpResponse, Self::Error>>;

  fn respond_to<S: 'static>(self, _req: &HttpRequest) -> Self::Future {
    ready(Ok(HttpResponse::NoContent().finish()))
  }
}

#[derive(Clone, Debug)]
pub struct Lines(Vec<String>);

impl Lines {
  pub fn new(lines: Vec<String>) -> Lines {
    Lines(lines)
  }
}
