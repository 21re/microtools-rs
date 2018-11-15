use actix_web::{HttpRequest, HttpResponse, Responder};
use problem::Problem;

#[derive(Clone, Debug)]
pub struct Done;

impl Responder for Done {
  type Item = HttpResponse;
  type Error = Problem;

  fn respond_to<S: 'static>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Problem> {
    Ok(HttpResponse::NoContent().finish())
  }
}

#[derive(Clone, Debug)]
pub struct Lines(Vec<String>);

impl Lines {
  pub fn new(lines: Vec<String>) -> Lines {
    Lines(lines)
  }
}
