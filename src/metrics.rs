use futures::Future;
use prometheus::{register, HistogramOpts, HistogramVec};
use std::time::Instant;

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
  /*  fn from_status(status: StatusCode) -> StatusCategory {
    match status.as_u16() {
      200..=299 => StatusCategory::Ok,
      300..=399 => StatusCategory::Redirect,
      400..=499 => StatusCategory::ClientError,
      _ => StatusCategory::InternalError,
    }
  }*/
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

  /*
  pub fn measure<S: 'static>(&self, r: &mut Resource<S>) {
    let pattern = r.rdef().pattern().to_string();
    r.middleware(MetricsMiddleware {
      histogram: self.histogram.clone(),
      path: pattern,
    })
  }*/
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

  pub async fn time_async<F, U, E>(&self, action: &'static str, f: F) -> Result<U, E>
  where
    F: Future<Output = Result<U, E>>,
  {
    let histogram = self.histogram.clone();
    let start = Instant::now();

    let result = f.await;

    let outcome = if result.is_ok() { "ok" } else { "err" };

    histogram
      .with_label_values(&[action, outcome])
      .observe(seconds_since(&start));
    result
  }

  pub fn time_sync<F, U, E>(&self, action: &str, f: F) -> Result<U, E>
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
