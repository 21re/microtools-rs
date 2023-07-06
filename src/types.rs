use actix_web::{HttpRequest, HttpResponse, Responder};

#[derive(Clone, Debug)]
pub struct Done;

impl Responder for Done {
  fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
    HttpResponse::NoContent().finish()
  }
}
