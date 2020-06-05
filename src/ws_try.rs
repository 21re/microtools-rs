use crate::{types::Done, AsyncBusinessResult, Problem};
use actix_web::client::{ClientResponse, PayloadError};
use actix_web::http::StatusCode;
use awc::SendClientRequest;
use bytes::Bytes;
use futures::{future, FutureExt, Stream, StreamExt};
use serde::de::DeserializeOwned;

const JSON_RESPONSE_LIMIT: usize = 100 * 1024 * 1024;

pub trait FromClientResponse<T> {
  fn from_response<S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static>(
    response: ClientResponse<S>,
  ) -> AsyncBusinessResult<T>;
}

impl FromClientResponse<Done> for Done {
  fn from_response<S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static>(
    response: ClientResponse<S>,
  ) -> AsyncBusinessResult<Done> {
    Box::pin(response.for_each(|_| future::ready(())).map(|_| Ok(Done)))
  }
}

impl<T> FromClientResponse<T> for T
where
  T: DeserializeOwned + 'static,
{
  fn from_response<S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static>(
    mut response: ClientResponse<S>,
  ) -> AsyncBusinessResult<T> {
    Box::pin(
      response
        .json()
        .limit(JSON_RESPONSE_LIMIT)
        .map(|r| r.map_err(Problem::from)),
    )
  }
}

pub trait ClientErrorHandler<T> {
  fn handle_error<S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static>(
    &self,
    response: ClientResponse<S>,
  ) -> AsyncBusinessResult<T>;
}

pub struct DefaultClientErrorHandler();

impl<T> ClientErrorHandler<T> for DefaultClientErrorHandler
where
  T: 'static,
{
  fn handle_error<S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static>(
    &self,
    response: ClientResponse<S>,
  ) -> AsyncBusinessResult<T> {
    Box::pin(future::err(Problem::for_status(
      response.status().as_u16(),
      format!("Service request failed: {}", response.status()),
    )))
  }
}

pub const DEFAULT_CLIENT_ERROR_HANDLER: DefaultClientErrorHandler = DefaultClientErrorHandler();

pub fn default_error_handler(status: StatusCode, _: Result<Bytes, PayloadError>) -> Problem {
  Problem::for_status(status.as_u16(), format!("Service request failed: {}", status))
}

pub trait SendClientRequestExt: Sized {
  fn expect_success<T>(self) -> AsyncBusinessResult<T>
  where
    T: FromClientResponse<T> + 'static,
  {
    self.expect_success_with_error(default_error_handler)
  }

  fn expect_success_with_error<T, E>(self, error_handler: E) -> AsyncBusinessResult<T>
  where
    T: FromClientResponse<T> + 'static,
    E: Fn(StatusCode, Result<Bytes, PayloadError>) -> Problem + 'static;
}

impl SendClientRequestExt for SendClientRequest {
  fn expect_success_with_error<T, E>(self, error_handler: E) -> AsyncBusinessResult<T>
  where
    T: FromClientResponse<T> + 'static,
    E: Fn(StatusCode, Result<Bytes, PayloadError>) -> Problem + 'static,
  {
    Box::pin(self.then(move |maybe_resp| match maybe_resp {
      Ok(mut resp) => {
        if resp.status().is_success() {
          T::from_response(resp)
        } else {
          Box::pin(resp.body().map(move |body| Err(error_handler(resp.status(), body))))
        }
      }
      Err(err) => Box::pin(future::err(Problem::from(err))),
    }))
  }
}
