use super::Problem;
use actix_web::dev::Resource;
use actix_web::error::Error;
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::{Finished, Middleware, Started};
use actix_web::{HttpRequest, HttpResponse};
use futures::Future;
use prometheus::{gather, register, Encoder, HistogramOpts, HistogramVec, TextEncoder};
use std::time::Instant;

pub fn metrics_resource<S: 'static>(r: &mut Resource<S>) {
  let encoder = TextEncoder::new();

  r.method(Method::GET).f(move |_| {
    let metrics = gather();
    let mut buffer = vec![];

    match encoder.encode(&metrics, &mut buffer) {
      Ok(_) => Ok(
        HttpResponse::build(StatusCode::OK)
          .content_type(encoder.format_type())
          .body(buffer),
      ),
      Err(err) => Err(Problem::internal_server_error().with_details(format!("{}", err))),
    }
  });
}

#[inline]
fn seconds_since(start: &Instant) -> f64 {
  let d = start.elapsed();
  let nanos = f64::from(d.subsec_nanos()) / 1e9;
  d.as_secs() as f64 + nanos
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StatusCategory {
  Ok,
  Redirect,
  ClientError,
  InternalError,
}

impl StatusCategory {
  fn from_status(status: StatusCode) -> StatusCategory {
    match status.as_u16() {
      200...299 => StatusCategory::Ok,
      300...399 => StatusCategory::Redirect,
      400...499 => StatusCategory::ClientError,
      _ => StatusCategory::InternalError,
    }
  }
}

impl StatusCategory {
  fn as_str(&self) -> &'static str {
    match self {
      StatusCategory::Ok => "2xx",
      StatusCategory::Redirect => "3xx",
      StatusCategory::ClientError => "4xx",
      StatusCategory::InternalError => "5xx",
    }
  }
}

#[derive(Clone)]
pub struct ResourceTimer {
  histogram: HistogramVec,
}

impl ResourceTimer {
  pub fn new<S: Into<String>>(name: S, help: S) -> ResourceTimer {
    let opts = HistogramOpts::new(name, help);
    let histogram = HistogramVec::new(opts, &["method", "path", "status"]).unwrap();

    register(Box::new(histogram.clone())).unwrap();

    ResourceTimer { histogram }
  }

  pub fn measure<S: 'static>(&self, r: &mut Resource<S>) {
    let pattern = r.rdef().pattern().to_string();
    r.middleware(MetricsMiddleware {
      histogram: self.histogram.clone(),
      path: pattern,
    })
  }
}

struct MetricsMiddleware {
  histogram: HistogramVec,
  path: String,
}

impl<S> Middleware<S> for MetricsMiddleware {
  fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
    req.extensions_mut().insert(Instant::now());
    Ok(Started::Done)
  }

  /// Method is called after body stream get sent to peer.
  fn finish(&self, req: &HttpRequest<S>, resp: &HttpResponse) -> Finished {
    if let Some(start) = req.extensions().get::<Instant>() {
      let method = req.method().as_str();
      let path = self.path.as_str();
      let status = StatusCategory::from_status(resp.status());
      self
        .histogram
        .with_label_values(&[method, path, status.as_str()])
        .observe(seconds_since(start));
    }

    Finished::Done
  }
}

#[derive(Clone)]
pub struct TimedActions {
  histogram: HistogramVec,
}

impl TimedActions {
  pub fn new<S: Into<String>>(name: S, help: S) -> TimedActions {
    let opts = HistogramOpts::new(name, help);
    let histogram = HistogramVec::new(opts, &["action", "outcome"]).unwrap();

    register(Box::new(histogram.clone())).unwrap();

    TimedActions { histogram }
  }

  pub fn async<F, U, E>(&self, action: &'static str, f: F) -> impl Future<Item = U, Error = E>
  where
    F: Future<Item = U, Error = E>,
  {
    let histogram = self.histogram.clone();
    let start = Instant::now();

    f.then(move |result| {
      let outcome = if result.is_ok() { "ok" } else { "err" };

      histogram
        .with_label_values(&[action, outcome])
        .observe(seconds_since(&start));
      result
    })
  }

  pub fn sync<F, U, E>(&self, action: &str, f: F) -> Result<U, E>
  where
    F: FnOnce() -> Result<U, E>,
  {
    let start = Instant::now();

    match f() {
      Ok(value) => {
        self
          .histogram
          .with_label_values(&[action, "ok"])
          .observe(seconds_since(&start));
        Ok(value)
      }
      Err(error) => {
        self
          .histogram
          .with_label_values(&[action, "err"])
          .observe(seconds_since(&start));
        Err(error)
      }
    }
  }
}
