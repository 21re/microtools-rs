use crate::problem::Problem;
use actix_web::{HttpRequest, HttpResponse, Responder};
use futures::future::{ok, Ready};

#[derive(Clone, Debug)]
pub struct Done;

impl Responder for Done {
  type Error = Problem;
  type Future = Ready<Result<HttpResponse, Self::Error>>;

  fn respond_to(self, _req: &HttpRequest) -> Self::Future {
    ok(HttpResponse::NoContent().finish())
  }
}
