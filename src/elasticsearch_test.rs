use super::elasticsearch::{Query, QueryRequest, SortOrder};
use serde_json;
use spectral::prelude::*;

#[test]
fn test_term_query() {
  let query = QueryRequest::new().with_query(Query::term("bla", "blub"));
  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"query":{"term":{"bla":"blub"}}}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}

#[test]
fn test_bool_query() {
  let query = QueryRequest::new().with_query(Query::bool_query(
    vec![Query::term("fieldname", "fieldvalue")], vec![], vec![],vec![],None)
  );

  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"query":{"bool":{"must":[{"term":{"fieldname":"fieldvalue"}}],"filter":[],"must_not":[],"should":[],"minimum_should_match":null}}}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}

#[test]
fn test_prefix_query() {
  let query = QueryRequest::new().with_query(Query::prefix("bla", "blub"));
  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"query":{"prefix":{"bla":"blub"}}}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}

#[test]
fn test_sort_without_query() {
  let query = QueryRequest::new()
    .with_sort("test1", SortOrder::Asc)
    .with_sort("test2", SortOrder::Desc);
  let json = serde_json::to_string(&query).unwrap();

  assert_that(&json.as_str()).is_equal_to(r#"{"sort":[{"test1":"asc"},{"test2":"desc"}]}"#);

  let actual = serde_json::from_str::<QueryRequest>(&json).unwrap();

  assert_that(&actual).is_equal_to(query);
}
