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

  pub fn status(&self) -> HttpResponse {
    HttpResponse::Ok().json(self)
  }
}

pub fn status_resource<V: ToString, S: 'static>(version: Option<V>) -> impl FnOnce(&mut Resource<S>) {
  let status = Status::new(version);

  web::resource("/status").route(web::get().to(move || HttpResponse::Ok().json(&status)))
}
