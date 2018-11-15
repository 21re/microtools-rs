use super::elasticsearch::{Query, QueryRequest};
use serde_json;
use spectral::prelude::*;

#[test]
fn test_term_query() {
  let query = QueryRequest::new(Query::term("bla", "blub"));
  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"query":{"term":{"bla":"blub"}}}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}

#[test]
fn test_prefix_query() {
  let query = QueryRequest::new(Query::prefix("bla", "blub"));
  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"query":{"prefix":{"bla":"blub"}}}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}
