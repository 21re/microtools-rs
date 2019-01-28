use super::elasticsearch::{ElasticsearchUrlBuilder, Query, QueryRequest, SortOrder};
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
    vec![Query::term("fieldname", "fieldvalue")],
    vec![],
    vec![],
    vec![],
    None,
  ));

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

#[test]
fn test_build_url_create() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.create("die id".to_string()))
    .is_equal_to("http://server/INDEX_NAME/_doc/die+id/_create?refresh=true".to_string());
}

#[test]
fn test_build_url_search() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.search()).is_equal_to("http://server/INDEX_NAME/_search".to_string());
}

#[test]
fn test_build_url_mget() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.mget()).is_equal_to("http://server/INDEX_NAME/_doc/_mget".to_string());
}

#[test]
fn test_build_url_mapping() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.mapping()).is_equal_to("http://server/INDEX_NAME/_mapping/_doc".to_string());
}

#[test]
fn test_build_url_update() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.update("die id".to_string()))
    .is_equal_to("http://server/INDEX_NAME/_doc/die+id/_update".to_string());
}

#[test]
fn test_build_url_delete_by_query() {
  let url_builder = ElasticsearchUrlBuilder::new("http://server".to_string(), "INDEX_NAME".to_string());

  assert_that(&url_builder.delete_by_query()).is_equal_to("http://server/INDEX_NAME/_delete_by_query".to_string());
}
