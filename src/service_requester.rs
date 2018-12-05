use super::types;
use actix::{Actor, Addr};
use actix_web::{client, http};
use futures::Future;
use gatekeeper;
use problem::Problem;
use serde::Serialize;
use url::form_urlencoded::byte_serialize;
use ws_try;

pub fn encode_url_component<S: AsRef<[u8]>>(value: S) -> String {
  byte_serialize(value.as_ref()).collect::<String>()
}

#[derive(Clone)]
pub struct ServiceRequester {
  token_creator: Addr<gatekeeper::TokenCreator>,
  error_handler: &'static (Fn(client::ClientResponse) -> Problem + Sync),
}

pub trait IntoClientRequest {
  fn apply_body(self, builder: &mut client::ClientRequestBuilder) -> Result<client::ClientRequest, Problem>;
}

impl ServiceRequester {
  pub fn with_service_auth(service_name: &str, scopes: &[(&str, &[&str])]) -> ServiceRequester {
    ServiceRequester {
      token_creator: gatekeeper::TokenCreator::for_service(service_name, scopes).start(),
      error_handler: &ws_try::default_error_handler,
    }
  }

  pub fn with_error_handler(&self, error_handler: &'static (Fn(client::ClientResponse) -> Problem + Sync)) -> Self {
    ServiceRequester {
      token_creator: self.token_creator.clone(),
      error_handler,
    }
  }

  #[inline]
  pub fn get<U, F, O>(&self, url: U) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    self.without_body(http::Method::GET, url)
  }

  #[inline]
  pub fn post<U, F, I, O>(&self, url: U, body: I) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    I: IntoClientRequest,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    self.with_body(http::Method::POST, url, body)
  }

  #[inline]
  pub fn put<U, F, I, O>(&self, url: U, body: I) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    I: IntoClientRequest,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    self.with_body(http::Method::PUT, url, body)
  }

  #[inline]
  pub fn patch<U, F, I, O>(&self, url: U, body: I) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    I: IntoClientRequest,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    self.with_body(http::Method::PATCH, url, body)
  }

  #[inline]
  pub fn delete<U, F>(&self, url: U) -> impl Future<Item = (), Error = Problem>
  where
    U: AsRef<str>,
  {
    self.without_body(http::Method::DELETE, url)
  }

  pub fn with_body<U, F, I, O>(&self, method: http::Method, url: U, body: I) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    I: IntoClientRequest,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    let error_handler_ref = self.error_handler;
    gatekeeper::get_token(&self.token_creator).and_then(move |token| {
      let request = body.apply_body(
        client::ClientRequest::build()
          .method(method)
          .uri(url)
          .header("Authorization", format!("Bearer {}", token.raw)),
      );

      ws_try::expect_success_with_error::<_, F, O, _>(request, error_handler_ref)
    })
  }

  pub fn without_body<U, F, O>(&self, method: http::Method, url: U) -> impl Future<Item = O, Error = Problem>
  where
    U: AsRef<str>,
    O: ws_try::FromClientResponse<Result = O, FutureResult = F>,
    F: Future<Item = O, Error = Problem>,
  {
    let error_handler_ref = self.error_handler;
    gatekeeper::get_token(&self.token_creator).and_then(move |token| {
      let request = client::ClientRequest::build()
        .method(method)
        .uri(url)
        .header("Authorization", format!("Bearer {}", token.raw))
        .finish();

      ws_try::expect_success_with_error::<_, F, O, _>(request, error_handler_ref)
    })
  }
}

impl<S> IntoClientRequest for S
where
  S: Serialize,
{
  fn apply_body(self, builder: &mut client::ClientRequestBuilder) -> Result<client::ClientRequest, Problem> {
    builder.json(self).map_err(Problem::from)
  }
}

impl IntoClientRequest for types::Done {
  fn apply_body(self, builder: &mut client::ClientRequestBuilder) -> Result<client::ClientRequest, Problem> {
    builder.finish().map_err(Problem::from)
  }
}
