use crate::{
  ws_try::{default_error_handler, FromClientResponse, SendClientRequestExt},
  BusinessResult, Problem,
};
use bytes::Bytes;
use reqwest::{redirect::Policy, Client, IntoUrl, Method, RequestBuilder, StatusCode};
use serde::Serialize;
use std::time::Duration;
use url::form_urlencoded::byte_serialize;

pub fn encode_url_component<S: AsRef<[u8]>>(value: S) -> String {
  byte_serialize(value.as_ref()).collect::<String>()
}

pub trait IntoClientRequest {
  fn apply_body(self, request: RequestBuilder) -> RequestBuilder;
}

impl<S> IntoClientRequest for S
where
  S: Serialize,
{
  fn apply_body(self, request: RequestBuilder) -> RequestBuilder {
    request.json(&self)
  }
}

#[derive(Clone)]
pub struct ServiceRequester {
  client: Client,
  service_name: &'static str,
  error_handler: &'static (dyn Fn(StatusCode, Result<Bytes, reqwest::Error>) -> Problem + Sync),
}

impl ServiceRequester {
  pub fn with_service_auth(service_name: &'static str) -> BusinessResult<Self> {
    ServiceRequester::with_service_auth_with_timeout(service_name, 120)
  }

  pub fn with_service_auth_with_timeout(service_name: &'static str, timeout_seconds: u16) -> BusinessResult<Self> {
    Ok(ServiceRequester {
      client: Client::builder()
        .connect_timeout(Duration::from_secs(timeout_seconds as u64))
        .timeout(Duration::from_secs(timeout_seconds as u64))
        .redirect(Policy::none())
        .build()?,
      service_name,
      error_handler: &default_error_handler,
    })
  }

  pub fn with_error_handler(
    self,
    error_handler: &'static (dyn Fn(StatusCode, Result<Bytes, reqwest::Error>) -> Problem + Sync),
  ) -> Self {
    ServiceRequester {
      client: self.client,
      service_name: self.service_name,
      error_handler,
    }
  }

  #[inline]
  pub async fn get<U, O>(&self, url: U) -> BusinessResult<O>
  where
    U: IntoUrl,
    O: FromClientResponse<O> + 'static,
  {
    self.without_body(Method::GET, url).await
  }

  #[inline]
  pub async fn post<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: IntoUrl,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::POST, url, body).await
  }

  #[inline]
  pub async fn patch<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: IntoUrl,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::PATCH, url, body).await
  }

  #[inline]
  pub async fn put<U, I, O>(&self, url: U, body: I) -> BusinessResult<O>
  where
    U: IntoUrl,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    self.with_body(Method::PUT, url, body).await
  }

  #[inline]
  pub async fn delete<U, O>(&self, url: U) -> BusinessResult<O>
  where
    U: IntoUrl,
    O: FromClientResponse<O> + 'static,
  {
    self.without_body(Method::DELETE, url).await
  }

  pub async fn with_body<U, I, O>(&self, method: Method, url: U, body: I) -> BusinessResult<O>
  where
    U: IntoUrl,
    I: IntoClientRequest,
    O: FromClientResponse<O> + 'static,
  {
    body
      .apply_body(
        self
          .client
          .request(method, url)
          .header("X-Auth-Sub", format!("service/{}", self.service_name))
          .header("X-Auth-Token", "internal-token"),
      )
      .expect_success_with_error(self.error_handler)
      .await
  }

  pub async fn without_body<U, O>(&self, method: Method, url: U) -> BusinessResult<O>
  where
    U: IntoUrl,
    O: FromClientResponse<O> + 'static,
  {
    self
      .client
      .request(method, url)
      .header("X-Auth-Sub", format!("service/{}", self.service_name))
      .header("X-Auth-Token", "internal-token")
      .expect_success_with_error(self.error_handler)
      .await
  }
}
