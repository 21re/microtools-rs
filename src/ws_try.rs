use actix_web::client;
use actix_web::HttpMessage;
use business_result::AsyncBusinessResult;
use futures::{future, Async, Future, Poll, Stream};
use problem::Problem;
use serde::de::DeserializeOwned;
use std::time::Duration;
use types::{Done, Lines};

const JSON_RESPONSE_LIMIT: usize = 20 * 1024 * 1024;

pub trait IntoClientRequest {
  fn into_request(self) -> Result<client::ClientRequest, Problem>;
}

pub trait FromClientResponse {
  type Result;
  type FutureResult: Future<Item = Self::Result, Error = Problem>;

  fn from_response(response: client::ClientResponse) -> Self::FutureResult;
}

pub enum WSTry<F> {
  MayBeSuccess(F),
  Failure(Problem),
  FutureFailure(Box<Future<Item = Problem, Error = Problem>>),
}

impl<T, F> Future for WSTry<F>
where
  F: Future<Item = T, Error = Problem>,
{
  type Item = T;
  type Error = Problem;

  fn poll(&mut self) -> Poll<T, Problem> {
    match self {
      WSTry::MayBeSuccess(f) => f.poll(),
      WSTry::Failure(problem) => Err(problem.clone()),
      WSTry::FutureFailure(future_problem) => match future_problem.poll() {
        Ok(Async::NotReady) => Ok(Async::NotReady),
        Ok(Async::Ready(problem)) => Err(problem),
        Err(problem) => Err(problem),
      },
    }
  }
}

pub fn try<R>(request: R) -> impl Future<Item = client::ClientResponse, Error = Problem>
where
  R: IntoClientRequest,
{
  let client_request = match request.into_request() {
    Ok(request) => request,
    Err(problem) => return WSTry::Failure(problem),
  };
  let url = client_request.uri().to_string();
  let method = client_request.method().to_string();

  WSTry::MayBeSuccess(
    client_request
      .send()
      .timeout(Duration::from_secs(60))
      .conn_timeout(Duration::from_secs(20))
      .wait_timeout(Duration::from_secs(60))
      .map_err(move |err| {
        error!("Request {} {} failed: {}", method, url, err);
        Problem::from(err)
      }),
  )
}

pub fn expect_success<R, F, T>(request: R) -> impl Future<Item = T, Error = Problem>
where
  R: IntoClientRequest,
  T: FromClientResponse<Result = T, FutureResult = F>,
  F: Future<Item = T, Error = Problem>,
{
  try(request).and_then(move |resp| {
    if resp.status().is_success() {
      WSTry::MayBeSuccess(T::from_response(resp))
    } else {
      WSTry::Failure(Problem::for_status(
        resp.status().as_u16(),
        format!("Service request failed: {}", resp.status()),
      ))
    }
  })
}

pub fn expect_success_with_error<R, F, T, E>(request: R, error_handler: E) -> impl Future<Item = T, Error = Problem>
where
  R: IntoClientRequest,
  T: FromClientResponse<Result = T, FutureResult = F>,
  F: Future<Item = T, Error = Problem>,
  E: Fn(client::ClientResponse) -> Box<Future<Item = Problem, Error = Problem>>,
{
  try(request).and_then(move |resp| {
    if resp.status().is_success() {
      WSTry::MayBeSuccess(T::from_response(resp))
    } else {
      WSTry::FutureFailure(error_handler(resp))
    }
  })
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub fn default_error_handler(response: client::ClientResponse) -> Box<Future<Item = Problem, Error = Problem>> {
  Box::new(future::ok(Problem::for_status(
    response.status().as_u16(),
    format!("Service request failed: {}", response.status()),
  )))
}

impl IntoClientRequest for client::ClientRequest {
  fn into_request(self) -> Result<client::ClientRequest, Problem> {
    Ok(self)
  }
}

impl IntoClientRequest for client::ClientRequestBuilder {
  fn into_request(mut self) -> Result<client::ClientRequest, Problem> {
    self.finish().map_err(Problem::from)
  }
}

impl<E> IntoClientRequest for Result<client::ClientRequest, E>
where
  E: Into<Problem>,
{
  fn into_request(self) -> Result<client::ClientRequest, Problem> {
    self.map_err(|err| err.into())
  }
}

impl<T> FromClientResponse for T
where
  T: DeserializeOwned + 'static,
{
  type Result = T;
  type FutureResult = AsyncBusinessResult<T>;

  fn from_response(response: client::ClientResponse) -> Self::FutureResult {
    Box::new(response.json().limit(JSON_RESPONSE_LIMIT).map_err(Problem::from))
  }
}

impl FromClientResponse for Done {
  type Result = Done;
  type FutureResult = AsyncBusinessResult<Done>;

  fn from_response(response: client::ClientResponse) -> Self::FutureResult {
    Box::new(response.payload().from_err().for_each(|_| Ok(())).map(|_| Done))
  }
}

impl FromClientResponse for Lines {
  type Result = Lines;
  type FutureResult = AsyncBusinessResult<Lines>;

  fn from_response(response: client::ClientResponse) -> Self::FutureResult {
    Box::new(response.readlines().collect().then(|result| match result {
      Ok(lines) => Ok(Lines::new(lines)),
      Err(error) => Err(Problem::from(error)),
    }))
  }
}
