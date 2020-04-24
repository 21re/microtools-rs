use super::types;
use crate::{gatekeeper, AsyncBusinessResult};
use crate::problem::Problem;
use crate::ws_try;
use actix::{Actor, Addr};
use actix_web::{client, http};
use futures::{Future, FutureExt};
use serde::Serialize;
use url::form_urlencoded::byte_serialize;
use actix_web::client::{Client, ClientRequest, ClientBuilder, ClientResponse, SendRequestError};
use std::convert::TryFrom;
use crate::gatekeeper::Token;
use crate::ws_try::WSTry;
use futures::future::TryFutureExt;
use std::pin::Pin;

pub fn encode_url_component<S: AsRef<[u8]>>(value: S) -> String {
  byte_serialize(value.as_ref()).collect::<String>()
}

#[derive(Clone)]
pub struct ServiceRequester {
  token_creator: Addr<gatekeeper::TokenCreator>,
  error_handler: &'static (dyn Fn(client::ClientResponse) -> Pin<Box<dyn Future<Output = Result<Problem, Problem>>>> + Sync),
}

pub trait IntoSendRequest {
  type Result;
  type FutureResult: Future<Output = Result<Self::Result, Problem>>;
  fn send(self, request: &mut client::ClientRequest) -> Self::FutureResult;
}

impl ServiceRequester {
  pub fn with_service_auth(service_name: &str, scopes: &[(&str, &[&str])]) -> ServiceRequester {
    ServiceRequester {
      token_creator: gatekeeper::TokenCreator::for_service(service_name, scopes).start(),
      error_handler: &ws_try::default_error_handler,
    }
  }

  pub fn with_error_handler(
    &self,
    error_handler: &'static (dyn Fn(client::ClientResponse) -> Pin<Box<dyn Future<Output = Result<Problem, Problem>>>> + Sync),
  ) -> Self {
    ServiceRequester {
      token_creator: self.token_creator.clone(),
      error_handler,
    }
  }

  #[inline]
  pub fn get<'a, U, F, O >(&'a self, url: String) -> impl Future<Output = Result<O, Problem>> + 'a
  where
    O: ws_try::FromClientResponse<Result = O, FutureResult = F> + 'a,
    F: Future<Output = Result<O, Problem>> + 'a,
  {
    self.without_body(http::Method::GET, url)
  }

  #[inline]
  pub fn post<'a, U, F, I, O>(&'a self, url: String, body: I) -> impl Future<Output = Result<O, Problem>> + 'a
  where
    I: IntoSendRequest + Serialize + 'a,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F> + 'a,
    F: Future<Output = Result<O, Problem>> + 'a,
  {
    self.with_body(http::Method::POST, url, body)
  }

  #[inline]
  pub fn put<'a, U, F, I, O>(&'a self, url: String, body: I) -> impl Future<Output = Result<O, Problem>> + 'a
  where
    I: IntoSendRequest + Serialize +'a,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F> +'a,
    F: Future<Output = Result<O, Problem>> +'a,
  {
    self.with_body(http::Method::PUT, url, body)
  }

  #[inline]
  pub fn patch<'a, U, F, I, O>(&'a self, url: String, body: I) -> impl Future<Output = Result<O, Problem>> +'a
  where
    I: IntoSendRequest + Serialize +'a,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F> +'a,
    F: Future<Output = Result<O, Problem>> +'a,
  {
    self.with_body(http::Method::PATCH, url, body)
  }

  #[inline]
  pub fn delete<'a, U, F, O>(&'a self, url: String) -> impl Future<Output = Result<O, Problem>> +'a
  where
    O: ws_try::FromClientResponse<Result = O, FutureResult = F> +'a,
    F: Future<Output = Result<O, Problem>> +'a,
  {
    self.without_body(http::Method::DELETE, url)
  }

  pub async fn with_body<F, I, O>(&self, method: http::Method, url: String, body: I) -> Result<O, Problem>
  where
    I: IntoSendRequest + Serialize,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Output = Result<O, Problem>>,
  {
    let error_handler_ref = self.error_handler;
    let token = gatekeeper::get_token(&self.token_creator).await;
    match token {
          Ok(tok) => {
            let mut request = Client::new().request(method, url)
                       .header("Authorization", format!("Bearer {}", tok.raw));

            let res = ws_try::expect_success_with_error_with_body::< F, O, _, I>(request, error_handler_ref, body).await;
            res
          },
          Err(e) => Err(e)
        }

  }

  pub async fn without_body<F, O>(&self, method: http::Method, url: String) -> Result<O, Problem>
    where
        O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
        F: Future<Output = Result<O, Problem>> ,
  {
    let error_handler_ref = self.error_handler;
    let token = gatekeeper::get_token(&self.token_creator).await;
    match token {
      Ok(tok) => {
        let mut request = Client::new().request(method, url)
            .header("Authorization", format!("Bearer {}", tok.raw));
        ws_try::expect_success_with_error::< F, O, _>(request, error_handler_ref).await
      },
      Err(e) => Err(e)
    }

  }
}

impl<S> IntoSendRequest for S
where
  S: Serialize,
{
  type Result = ClientResponse;
  type FutureResult = AsyncBusinessResult<Self::Result>;

  fn send(self, request: &mut client::ClientRequest) -> Self::FutureResult {
    Box::pin(request.send_json(&self).map_err(Problem::from))
  }
}

impl IntoSendRequest for types::Done {
  type Result = ClientResponse;
  type FutureResult = AsyncBusinessResult<Self::Result>;

  fn send(self, request: &mut client::ClientRequest) -> Self::FutureResult {
    Box::pin(request.send().map_err(Problem::from))
  }
}
