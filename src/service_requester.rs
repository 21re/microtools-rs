use crate::{
  gatekeeper::{get_token, TokenCreator},
  ws_try::{default_error_handler, FromClientResponse, SendClientRequestExt},
  BusinessResult, BusinessResultExt, Problem,
};
use actix::{Actor, Addr};
use actix_web::client::{Client, ClientRequest, PayloadError};
use actix_web::http::{Method, StatusCode, Uri};
use awc::{Connector, SendClientRequest};
use bytes::Bytes;
use serde::Serialize;
use std::{convert::TryInto, time::Duration};
use url::form_urlencoded::byte_serialize;

pub fn encode_url_component<S: AsRef<[u8]>>(value: S) -> String {
  byte_serialize(value.as_ref()).collect::<String>()
}

pub trait IntoClientRequest {
  fn apply_body(self, request: ClientRequest) -> SendClientRequest;
}

impl<S> IntoClientRequest for S
where
  S: Serialize,
{
  fn apply_body(self, request: ClientRequest) -> SendClientRequest {
    request.send_json(&self)
  }
}

#[derive(Clone)]
pub struct ServiceRequester {
  token_creator: Addr<TokenCreator>,
  error_handler: &'static (dyn Fn(StatusCode, Result<Bytes, PayloadError>) -> Problem + Sync),
}

impl ServiceRequester {
  pub fn with_service_auth(service_name: &str, scopes: &[(&str, &[&str])]) -> ServiceRequester {
    ServiceRequester {
      token_creator: TokenCreator::for_service(service_name, scopes).start(),
      error_handler: &default_error_handler,
    }
  }

  pub fn with_error_handler(
    self,
    error_handler: &'static (dyn Fn(StatusCode, Result<Bytes, PayloadError>) -> Problem + Sync),
  ) -> Self {
    ServiceRequester {
      token_creator: self.token_creator,
      error_handler,
    }
  }

  #[inline]
  pub async fn get<U, O>(&self, url: U) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    O: FromClientResponse<O> + 'static,
  {
    self.without_body(Method::GET, url).await
  }

  #[inline]
  pub async fn post<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::POST, url, body).await
  }

  #[inline]
  pub async fn patch<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::PATCH, url, body).await
  }

  #[inline]
  pub async fn put<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::PUT, url, body).await
  }

  #[inline]
  pub async fn delete<U, O>(&self, url: U) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    O: FromClientResponse<O> + 'static,
  {
    self.without_body(Method::DELETE, url).await
  }

  pub async fn with_body<U, I, O>(&self, method: Method, url: U, body: I) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    let client = Client::build().disable_redirects().connector(Connector::new().timeout(Duration::from_secs(20)).finish()).finish();
    let token = get_token(&self.token_creator).await?;

    body
      .apply_body(
        client
          .request(method, url.try_into().chain_problem("Invalid uri")?)
          .header("Authorization", format!("Bearer {}", token.raw))
          .timeout(Duration::from_secs(60)),
      )
      .expect_success_with_error(self.error_handler)
      .await
  }

  pub async fn without_body<U, O>(&self, method: Method, url: U) -> BusinessResult<O>
  where
    U: TryInto<Uri>,
    O: FromClientResponse<O> + 'static,
  {
    let client = Client::build().disable_redirects().connector(Connector::new().timeout(Duration::from_secs(20)).finish()).finish();
    let token = get_token(&self.token_creator).await?;

    client
      .request(method, url.try_into().chain_problem("Invalid uri")?)
      .header("Authorization", format!("Bearer {}", token.raw))
      .timeout(Duration::from_secs(60))
      .send()
      .expect_success_with_error(self.error_handler)
      .await
  }
}
