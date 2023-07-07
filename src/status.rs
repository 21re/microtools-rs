use std::sync::Arc;

use actix_web::{web, HttpResponse, Resource, Responder};
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

async fn status_handler(status: web::Data<Arc<Status>>) -> impl Responder {
  let status = status.as_ref();
  HttpResponse::Ok().json(status)
}

pub fn status_resource<V: ToString>(version: Option<V>) -> Resource {
  let status = Status::new(version);
  let shared_status = Arc::new(status);

  web::resource("/status")
    .app_data(shared_status.clone())
    .route(web::get().to(status_handler))
}
