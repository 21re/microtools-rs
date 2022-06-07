use super::{AsyncBusinessResult, BusinessResult, Problem};
use crate::subject::Subject;
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::FromRequest;
use actix_web::{
  dev::{MessageBody, Payload, Service, ServiceRequest, ServiceResponse, Transform},
  HttpMessage, HttpRequest, HttpResponse, Result,
};
use futures::future::{err, ok, Future, Ready};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::task::{Context, Poll};

#[derive(Clone, Debug)]
pub struct AuthContext {
  subject: Subject,
  token: String,
  organization: Option<String>,
  scopes: BTreeMap<String, Vec<String>>,
}

impl AuthContext {
  pub async fn require<R, F, FU, U>(self, requirements: R, f: F) -> BusinessResult<U>
  where
    F: FnOnce() -> FU,
    FU: Future<Output = BusinessResult<U>>,
    R: FnOnce(&AuthContext) -> bool,
    U: 'static,
  {
    if requirements(&self) {
      f().await
    } else {
      Err(Problem::forbidden())
    }
  }
}

pub fn admin_scope(auth_context: &AuthContext) -> bool {
  matches!(auth_context.subject, Subject::Admin(_))
}

impl FromRequest for AuthContext {
  type Error = Problem;
  type Future = Ready<Result<AuthContext, Problem>>;
  type Config = ();

  fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
    match req.extensions().get::<AuthContext>() {
      Some(auth_context) => ok(auth_context.to_owned()),
      None => err(Problem::unauthorized()),
    }
  }
}

static SUBJECT_HEADER_NAME: &str = "x-auth-sub";
static TOKEN_HEADER_NAME: &str = "x-auth-token";
static ORGANIZATION_HEADER_NAME: &str = "x-auth-org";
static SCOPES_HEADER_PREFIX: &str = "x-auth-scopes-";

pub fn admin_scoped_action<F>(req: &HttpRequest, f: F) -> Result<HttpResponse>
where
  F: Fn(AuthContext) -> Result<HttpResponse>,
{
  match req.extensions().get::<AuthContext>() {
    Some(auth_context) => match auth_context.subject {
      Subject::Admin(_) => f(auth_context.to_owned()),
      _ => Err(ErrorForbidden("Only admins are allowed")),
    },
    _ => Err(ErrorUnauthorized("")),
  }
}

pub fn customer_scoped_action<F>(req: &HttpRequest, f: F) -> Result<HttpResponse>
where
  F: Fn(AuthContext) -> Result<HttpResponse>,
{
  match req.extensions().get::<AuthContext>() {
    Some(auth_context) => match auth_context.subject {
      Subject::Customer(_) => f(auth_context.to_owned()),
      _ => Err(ErrorForbidden("Only customers are allowed")),
    },
    _ => Err(ErrorUnauthorized("")),
  }
}

pub fn service_scoped_action<F>(req: &HttpRequest, f: F) -> Result<HttpResponse>
where
  F: Fn(AuthContext) -> Result<HttpResponse>,
{
  match req.extensions().get::<AuthContext>() {
    Some(auth_context) => match auth_context.subject {
      Subject::Service(_) => f(auth_context.to_owned()),
      _ => Err(ErrorForbidden("Only services are allowed")),
    },
    _ => Err(ErrorUnauthorized("")),
  }
}

fn extract_scopes_from_headers(headers: &HeaderMap) -> BTreeMap<String, Vec<String>> {
  let mut scopes: BTreeMap<String, Vec<String>> = BTreeMap::new();

  headers
    .iter()
    .filter(|(n, _)| n.as_str().to_lowercase().starts_with(SCOPES_HEADER_PREFIX))
    .for_each(|(n, v)| {
      let service = n.as_str().to_lowercase().replace(SCOPES_HEADER_PREFIX, "");

      if let Ok(value) = v.to_str() {
        let mut existing_scopes = scopes.get_mut(&service).unwrap_or(&mut vec![]).to_vec();
        existing_scopes.push(value.to_string());

        scopes.insert(service, existing_scopes);
      };
    });

  scopes
}

fn extract_organization(maybe_organization: Option<&HeaderValue>) -> Option<String> {
  match maybe_organization {
    Some(organization) => match organization.to_str() {
      Ok(organization) => Some(organization.to_string()),
      Err(_) => None,
    },
    None => None,
  }
}

pub struct AuthMiddlewareFactory();

impl<S, B> Transform<S> for AuthMiddlewareFactory
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Problem> + 'static,
  B: MessageBody,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Problem;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(AuthMiddleware { service })
  }
}

impl Default for AuthMiddlewareFactory {
  fn default() -> AuthMiddlewareFactory {
    AuthMiddlewareFactory()
  }
}

pub struct AuthMiddleware<S> {
  service: S,
}

impl<S, B> Service for AuthMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Problem> + 'static,
  B: MessageBody,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Problem;
  type Future = AsyncBusinessResult<Self::Response>;

  fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, req: ServiceRequest) -> Self::Future {
    let maybe_subject: Option<&HeaderValue> = req.headers().get(SUBJECT_HEADER_NAME);
    let maybe_token: Option<&HeaderValue> = req.headers().get(TOKEN_HEADER_NAME);
    let maybe_organization: Option<&HeaderValue> = req.headers().get(ORGANIZATION_HEADER_NAME);

    if let (Some(subject), Some(token)) = (maybe_subject, maybe_token) {
      let scopes: BTreeMap<String, Vec<String>> = extract_scopes_from_headers(req.headers());

      if let (Ok(subject_str), Ok(token)) = (subject.to_str(), token.to_str()) {
        if let Ok(subject) = Subject::from_str(subject_str) {
          req.extensions_mut().insert(AuthContext {
            subject,
            token: token.to_string(),
            organization: extract_organization(maybe_organization),
            scopes,
          });
        }
      }
    }

    let fut = self.service.call(req);

    Box::pin(async move {
      let res = fut.await?;
      Ok(res)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::http::header::HeaderName;
  use spectral::prelude::*;

  #[test]
  fn extract_organization_from_header_is_successful() {
    let header_value = HeaderValue::from_static("fkbr org");

    assert_that(&extract_organization(Option::from(&header_value))).is_equal_to(Some("fkbr org".to_string()))
  }

  #[test]
  fn extract_scopes_from_empty_header_map() {
    let headers = HeaderMap::new();

    assert_that(&extract_scopes_from_headers(&headers)).is_equal_to(BTreeMap::new())
  }

  #[test]
  fn extract_scopes_is_successful() {
    let mut headers = HeaderMap::new();
    headers.append(HeaderName::from_static("x-auth-scopes-kuci"), "sxoe".parse().unwrap());
    headers.append(HeaderName::from_static("x-auth-scopes-kuci"), "fkbr".parse().unwrap());
    headers.append(HeaderName::from_static("x-auth-scopes-sxoe"), "kuci".parse().unwrap());

    let scopes = extract_scopes_from_headers(&headers);

    assert_that(&scopes.get("kuci").unwrap()).is_equal_to(&vec!["fkbr".to_string(), "sxoe".to_string()]);
    assert_that(&scopes.get("sxoe").unwrap()).is_equal_to(&vec!["kuci".to_string()]);
  }
}
