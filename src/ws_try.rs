use crate::business_result::AsyncBusinessResult;
use crate::problem::Problem;
use crate::types::{Done, Lines};
use actix_web::client;
use actix_web::client::ClientResponse;
use actix_web::HttpMessage;
use futures::{future, Future, Stream};
use log::error;
use serde::de::DeserializeOwned;
use std::time::Duration;
use futures::task::{Poll, Context};
use std::pin::Pin;

const JSON_RESPONSE_LIMIT: usize = 100 * 1024 * 1024;

pub trait IntoClientRequest {
  fn into_request(self) -> Result<client::ClientRequest, Problem>;
}

pub trait FromClientResponse {
  type Result;
  type FutureResult: Future<Output = Self::Result>;

  fn from_response(response: &client::ClientResponse) -> Self::FutureResult;
}

pub enum WSTry<F> {
  MayBeSuccess(F),
  Failure(Problem),
  FutureFailure(Box<dyn Future<Output = Problem>>),
}

impl<T, F> Future for WSTry<F>
where
  F: Future<Output = T>,
{

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match self.state {
      WSTry::MayBeSuccess(f) =>  f.poll(),
      WSTry::Failure(problem) => Poll::Ready(Err(problem.clone())),
      WSTry::FutureFailure(future_problem) => match future_problem.poll() {
        Poll::Pending => Poll::Pending,
        Poll::Ready(problem) => Poll::Ready(Err(problem.clone())),
      },
    }
  }

}

pub fn r#try<R>(request: R) -> impl Future<Item = client::ClientResponse, Error = Problem>
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
  F: Future<Output = T>,
{
  r#try(request).and_then(move |resp| {
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

pub fn expect_success_with_error<R, F, T, E>(request: R, error_handler: E) -> impl Future<Output = T>
where
  R: IntoClientRequest,
  T: FromClientResponse<Result = T, FutureResult = F>,
  F: Future<Output = T>,
  E: Fn(client::ClientResponse) -> Box<dyn Future<Output = Problem>>,
{
  r#try(request).and_then(move |resp| {
    if resp.status().is_success() {
      WSTry::MayBeSuccess(T::from_response(resp))
    } else {
      WSTry::FutureFailure(error_handler(resp))
    }
  })
}

#[allow(clippy::needless_pass_by_value)]
pub fn default_error_handler(response: client::ClientResponse) -> Box<dyn Future<Output = Problem>> {
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

impl IntoClientRequest for client::ClientBuilder {
  fn into_request(mut self) -> Result<client::ClientRequest, Problem> {
    self.finish().map_err(Problem::from)
  }
}

impl<E> IntoClientRequest for Result<client::ClientRequest, E>
where
  E: Into<Problem>,
{
  fn into_request(self) -> Result<client::ClientRequest, Problem> {
    self.map_err(E::into)
  }
}
//
// impl<T> FromClientResponse for T
// where
//   T: DeserializeOwned + 'static,
// {
//   type Result = T;
//   type FutureResult = AsyncBusinessResult<T>;
//
//   fn from_response(mut response: actix_web::client::ClientResponse) -> Self::FutureResult {
//     Box::new(response.json().limit(JSON_RESPONSE_LIMIT).map_err(Problem::from))
//   }
// }

// impl FromClientResponse for Done {
//   type Result = Done;
//   type FutureResult = AsyncBusinessResult<Done>;
//
//   fn from_response(response: client::ClientResponse) -> Self::FutureResult {
//     Box::new(response.payload().from_err().for_each(|_| Ok(())).map(|_| Done))
//   }
// }
//
// impl FromClientResponse for Lines {
//   type Result = Lines;
//   type FutureResult = AsyncBusinessResult<Lines>;
//
//   fn from_response(response: client::ClientResponse) -> Self::FutureResult {
//     Box::new(response.readlines().collect().then(|result| match result {
//       Ok(lines) => Ok(Lines::new(lines)),
//       Err(error) => Err(Problem::from(error)),
//     }))
//   }
// }
