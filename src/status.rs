use actix_web::{web, HttpResponse, Resource};
use serde_derive::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Status {
  version: String,
}

impl Status {
  pub fn new<S: ToString>(version: Option<S>) -> Status {
    Status {
      version: version.map(|s| s.to_string()).unwrap_or_else(|| "UNKNOWN".to_string()),
    }
  }

  pub async fn status(&self) -> HttpResponse {
    HttpResponse::Ok().json(self)
  }
}

pub fn status_resource<V: ToString>(version: Option<V>) -> Resource {
  let status = Status::new(version);

  web::resource("/status").route(web::get().to(|| status.status()))
}
