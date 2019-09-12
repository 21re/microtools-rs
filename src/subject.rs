use actix_web::Result;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Subject {
  Admin(String),
  Customer(String),
  Api(String),
  Service(String),
  Generic(String),
}

static ADMIN_SUBJECT: &str = "admin/";
static CUSTOMER_SUBJECT: &str = "customer/";
static SERVICE_SUBJECT: &str = "service/";
static API_SUBJECT: &str = "api/";

impl FromStr for Subject {
  type Err = ::actix_web::error::Error;

  fn from_str(subject: &str) -> Result<Subject> {
    match subject {
      subject if subject.starts_with(ADMIN_SUBJECT) => Ok(Subject::Admin(subject.replace(ADMIN_SUBJECT, ""))),
      subject if subject.starts_with(CUSTOMER_SUBJECT) => Ok(Subject::Customer(subject.replace(CUSTOMER_SUBJECT, ""))),
      subject if subject.starts_with(SERVICE_SUBJECT) => Ok(Subject::Service(subject.replace(SERVICE_SUBJECT, ""))),
      subject if subject.starts_with(API_SUBJECT) => Ok(Subject::Api(subject.replace(API_SUBJECT, ""))),
      subject => Ok(Subject::Generic(subject.to_string())),
    }
  }
}

impl ToString for Subject {
  fn to_string(&self) -> String {
    match self {
      Subject::Admin(subject) => format!("{}{}", ADMIN_SUBJECT, subject),
      Subject::Customer(subject) => format!("{}{}", CUSTOMER_SUBJECT, subject),
      Subject::Service(subject) => format!("{}{}", SERVICE_SUBJECT, subject),
      Subject::Api(subject) => format!("{}{}", API_SUBJECT, subject),
      Subject::Generic(subject) => subject.to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use spectral::prelude::*;

  #[test]
  fn subject_marshalling_works() {
    assert_that(&Subject::Admin("foo".to_string()).to_string()).is_equal_to("admin/foo".to_string());
    assert_that(&Subject::Customer("foo".to_string()).to_string()).is_equal_to("customer/foo".to_string());
    assert_that(&Subject::Service("foo".to_string()).to_string()).is_equal_to("service/foo".to_string());
    assert_that(&Subject::Api("foo".to_string()).to_string()).is_equal_to("api/foo".to_string());
    assert_that(&Subject::Generic("foo".to_string()).to_string()).is_equal_to("foo".to_string());
  }
}
