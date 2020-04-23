use crate::business_result::AsyncBusinessResult;
use crate::problem::Problem;
use crate::types::{Done, Lines};
use actix_web::client;
use actix_web::client::{Client, ClientResponse, ClientRequest, SendRequestError};
use actix_web::HttpMessage;
use futures::{future, Future, Stream, StreamExt};
use log::error;
use serde::de::DeserializeOwned;
use std::time::Duration;
use futures::task::{Poll, Context};
use std::pin::Pin;
use futures::future::TryFutureExt;
use serde::Serialize;
use crate::{IntoSendRequest, BusinessResult};
use actix_web::dev::{PayloadStream, Payload};

const JSON_RESPONSE_LIMIT: usize = 100 * 1024 * 1024;


pub trait FromClientResponse {
  type Result;
  type FutureResult: Future<Output = Result<Self::Result, Problem>>;

  fn from_response(response: &client::ClientResponse) -> Self::FutureResult;
}

pub enum WSTry<F> {
  MayBeSuccess(F),
  Failure(Problem),
  FutureFailure(Box<dyn Future<Output = Problem>>),
}

// impl<T, F> Future for WSTry<F>
//
// where
//   F: Future<Output = T> + Unpin,
// {
//   type Output = Result<T, Problem>;
//
//   fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//     match self.get_mut() {
//       WSTry::MayBeSuccess(f) =>  f.into_future().poll(),
//       WSTry::Failure(problem) => Poll::Ready(Err(problem.clone())),
//       WSTry::FutureFailure(future_problem) => match future_problem.poll() {
//         Poll::Pending => Poll::Pending,
//         Poll::Ready(problem) => Poll::Ready(Err(problem.clone())),
//       },
//     }
//   }
//
// }

pub async fn try_without_body(request: ClientRequest) -> BusinessResult<ClientResponse> {

  let url = request.get_uri();
  let method = request.get_method();

    request
      .timeout(Duration::from_secs(60))
      .send()
      .map_err(move |err| {
        error!("Request {} {} failed: {}", method, url, err);
        Problem::from(err)
      }).await
}

pub async fn try_with_body<B>(request: ClientRequest, body: B) -> Result<client::ClientResponse, Problem>
  where
      B: IntoSendRequest + Serialize,
{

  let url = request.get_uri();
  let method = request.get_method();

    request
        .timeout(Duration::from_secs(60))
        .send_json(&body)
        .map_err(move |err| {
          error!("Request {} {} failed: {}", method, url, err);
          Problem::from(err)
        }).await
}

pub async fn expect_success<F, T>(request: ClientRequest) -> Result<T, Problem>
where
  T: FromClientResponse<Result = T, FutureResult = F>,
  F: Future<Output = Result<T, Problem>>,
{
  match try_without_body(request).await {
    Ok(resp) if resp.status().is_success() =>
      T::from_response(&resp).await,
    Ok(resp) => Err(Problem::internal_server_error().with_details("expect success")),
    Err(e) => Err(e),
  }
}

pub async fn expect_success_with_error_with_body<F, T, E, B>(request: ClientRequest, error_handler: E, body: B) -> Result<T, Problem>
  where
      F: Future<Output = Result<T, Problem>>,
      T: FromClientResponse<Result = T, FutureResult = F>,
      E: Fn(client::ClientResponse) -> Pin<Box<dyn Future<Output = Result<Problem, ()>>>>,
      B: IntoSendRequest + Serialize,
{
  match try_with_body(request, body).await {
    Ok(resp) if resp.status().is_success() =>
      T::from_response(&resp).await,
    Ok(resp) => {
      error_handler(resp).await
      Err()
    },
    Err(e) => Err(e),
  }
}


pub async fn expect_success_with_error<F, T, E>(request: ClientRequest, error_handler: E) -> Result<T, Problem>
where
  T: FromClientResponse<Result = T, FutureResult = F>,
  F: Future<Output = Result<T, Problem>>,
  E: Fn(client::ClientResponse) -> Pin<Box<dyn Future<Output = Result<Problem, ()>>>>,
{
  match try_without_body(request).await {
    Ok(resp) if resp.status().is_success() =>
      T::from_response(&resp).await,
    Ok(resp) => Err(error_handler(resp).await),
    Err(e) => Err(e),
    }
  }


#[allow(clippy::needless_pass_by_value)]
pub fn default_error_handler(response: client::ClientResponse) -> Pin<Box<dyn Future<Output = Result<(), Problem>>>> {
  Box::pin(future::ok(Some(Problem::for_status(
    response.status().as_u16(),
    format!("Service request failed: {}", response.status()),
  ))))
}


impl<T> FromClientResponse for T
where
  T: DeserializeOwned + 'static,
{
  type Result = T;
  type FutureResult = AsyncBusinessResult<T>;

  fn from_response(mut response: &client::ClientResponse) -> Self::FutureResult {
    Box::pin( response.json().limit(JSON_RESPONSE_LIMIT).map_err(Problem::from))


  }
}

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
