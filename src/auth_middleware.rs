use super::{business_result, AsyncBusinessResult, BusinessResult, Problem};
use crate::subject::Subject;
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::middleware::{Middleware, Started};
use actix_web::FromRequest;
use actix_web::{HttpRequest, HttpResponse, Result};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct AuthContext {
  subject: Subject,
  token: String,
  organization: Option<String>,
  scopes: BTreeMap<String, Vec<String>>,
}

impl AuthContext {
  pub fn require<R, F, U>(self, requirements: R, f: F) -> AsyncBusinessResult<U>
  where
    F: FnOnce() -> AsyncBusinessResult<U>,
    R: FnOnce(&AuthContext) -> bool,
    U: 'static,
  {
    if requirements(&self) {
      f()
    } else {
      business_result::failure(Problem::forbidden())
    }
  }
}

pub fn admin_scope(auth_context: &AuthContext) -> bool {
  match auth_context.subject {
    Subject::Admin(_) => true,
    _ => false,
  }
}

impl<S> FromRequest<S> for AuthContext {
  type Config = ();
  type Result = BusinessResult<AuthContext>;

  fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
    match req.extensions().get::<AuthContext>() {
      Some(auth_context) => Ok(auth_context.to_owned()),
      None => Err(Problem::unauthorized()),
    }
  }
}

static SUBJECT_HEADER_NAME: &str = "X-Auth-Sub";
static TOKEN_HEADER_NAME: &str = "X-Auth-Token";
static ORGANIZATION_HEADER_NAME: &str = "X-Auth-Org";
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

pub struct AuthMiddleware {}

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

impl<S> Middleware<S> for AuthMiddleware {
  fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
    let maybe_subject: Option<&HeaderValue> = req.headers().get(SUBJECT_HEADER_NAME);
    let maybe_token: Option<&HeaderValue> = req.headers().get(TOKEN_HEADER_NAME);
    let maybe_organization: Option<&HeaderValue> = req.headers().get(ORGANIZATION_HEADER_NAME);

    if let (Some(subject), Some(token)) = (maybe_subject, maybe_token) {
      let scopes: BTreeMap<String, Vec<String>> = extract_scopes_from_headers(req.headers());

      if let (Ok(subject), Ok(token)) = (subject.to_str(), token.to_str()) {
        req.extensions_mut().insert(AuthContext {
          subject: Subject::from_str(subject)?,
          token: token.to_string(),
          organization: extract_organization(maybe_organization),
          scopes,
        });
      }
    }

    Ok(Started::Done)
  }
}

impl Default for AuthMiddleware {
  fn default() -> AuthMiddleware {
    AuthMiddleware {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;
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
    headers.append("X-Auth-Scopes-Kuci", "fkbr".parse().unwrap());
    headers.append("X-Auth-Scopes-Kuci", "sxoe".parse().unwrap());
    headers.append("X-Auth-Scopes-Sxoe", "kuci".parse().unwrap());

    let scopes = extract_scopes_from_headers(&headers);

    assert_that(&scopes.get("kuci").unwrap()).is_equal_to(&vec!["fkbr".to_string(), "sxoe".to_string()]);
    assert_that(&scopes.get("sxoe").unwrap()).is_equal_to(&vec!["kuci".to_string()]);
  }
}
