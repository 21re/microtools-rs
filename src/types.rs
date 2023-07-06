use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};

#[derive(Clone, Debug)]
pub struct Done;

impl Responder for Done {
  fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
    HttpResponse::NoContent().finish()
  }

  type Body = BoxBody;
}
