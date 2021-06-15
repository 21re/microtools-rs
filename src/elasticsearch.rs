use super::encode_url_component;
use super::serde_field_value;
use super::{IntoClientRequest, Problem};
use bytes::Bytes;
use futures::stream;
use reqwest::{header, Body, RequestBuilder};
use serde::de::{MapAccess, Visitor};
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use serde_json::to_writer;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
  Bool(bool),
  Int(i64),
  String(String),
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Value::Bool(b) => write!(f, "{}", b),
      Value::Int(i) => write!(f, "{}", i),
      Value::String(str) => write!(f, "{}", str),
    }
  }
}

impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Value::Bool(b)
  }
}

impl From<i64> for Value {
  fn from(i: i64) -> Self {
    Value::Int(i)
  }
}

impl<'a> From<&'a str> for Value {
  fn from(s: &'a str) -> Self {
    Value::String(s.to_string())
  }
}

impl From<String> for Value {
  fn from(s: String) -> Self {
    Value::String(s)
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct QueryRequest {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub sort: Vec<Sort>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub query: Option<Query>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub from: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size: Option<u64>,
}

impl QueryRequest {
  pub fn new() -> QueryRequest {
    QueryRequest {
      sort: Vec::new(),
      query: None,
      from: None,
      size: None,
    }
  }

  pub fn with_query(mut self, query: Query) -> Self {
    self.query = Some(query);
    self
  }

  pub fn with_sort<F: Into<String>>(mut self, field: F, order: SortOrder) -> Self {
    self.sort.push(Sort::new(field, order));
    self
  }

  pub fn with_from(mut self, from: u64) -> Self {
    self.from = Some(from);
    self
  }

  pub fn with_size(mut self, size: u64) -> Self {
    self.size = Some(size);
    self
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Query {
  Bool(BoolQuery),
  #[serde(with = "serde_field_value")]
  Term(String, Value),
  #[serde(with = "serde_field_value")]
  Prefix(String, Value),
}

impl Query {
  pub fn term<F: Into<String>, V: Into<Value>>(field: F, value: V) -> Query {
    Query::Term(field.into(), value.into())
  }

  pub fn prefix<F: Into<String>, V: Into<Value>>(field: F, value: V) -> Query {
    Query::Prefix(field.into(), value.into())
  }

  pub fn bool_query(
    must: Vec<Query>,
    filter: Vec<Query>,
    must_not: Vec<Query>,
    should: Vec<Query>,
    minimum_should_match: Option<i32>,
  ) -> Query {
    Query::Bool(BoolQuery {
      must,
      filter,
      must_not,
      should,
      minimum_should_match,
    })
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BoolQuery {
  #[serde(default)]
  pub must: Vec<Query>,
  #[serde(default)]
  pub filter: Vec<Query>,
  #[serde(default)]
  pub must_not: Vec<Query>,
  #[serde(default)]
  pub should: Vec<Query>,
  pub minimum_should_match: Option<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sort {
  field: String,
  order: SortOrder,
}

impl Sort {
  fn new<F: Into<String>>(field: F, order: SortOrder) -> Sort {
    Sort {
      field: field.into(),
      order,
    }
  }
}

impl serde::Serialize for Sort {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry(&self.field, &self.order)?;
    map.end()
  }
}

impl<'de> serde::Deserialize<'de> for Sort {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct FieldVisitor;

    impl<'de> Visitor<'de> for FieldVisitor {
      type Value = Sort;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("sort object")
      }

      fn visit_map<V>(self, mut map: V) -> Result<Sort, V::Error>
      where
        V: MapAccess<'de>,
      {
        if let Some(field) = map.next_key()? {
          return Ok(Sort {
            field,
            order: map.next_value()?,
          });
        }
        Err(::serde::de::Error::missing_field("field"))
      }
    }

    deserializer.deserialize_map(FieldVisitor)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
  Asc,
  Desc,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryResult<I, T> {
  pub hits: QueryHits<I, T>,
  #[serde(rename = "_scroll_id")]
  pub scroll_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryHits<I, T> {
  pub total: QueryHitsTotal,
  pub hits: Vec<Doc<I, T>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryHitsTotal {
  pub total: u64,
  pub relation: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Doc<I, T> {
  #[serde(rename = "_id")]
  pub id: I,
  #[serde(rename = "_source")]
  pub source: T,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MGetResult<T> {
  pub docs: Vec<MGetDoc<T>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MGetDoc<T> {
  #[serde(rename = "_id")]
  pub id: String,
  pub found: bool,
  #[serde(rename = "_source")]
  pub source: Option<T>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AggregationsResponse<T> {
  pub aggregations: HashMap<String, T>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BucketAggregation {
  pub buckets: Vec<Bucket>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Bucket {
  pub key: Value,
  pub key_as_string: Option<String>,
  pub doc_count: u64,
  #[serde(flatten)]
  pub children: HashMap<String, BucketAggregation>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulkActionResponse {
  #[serde(rename = "_id")]
  pub id: String,
  pub status: u16,
  pub result: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulkResponseItem {
  pub index: Option<BulkActionResponse>,
  pub update: Option<BulkActionResponse>,
  pub delete: Option<BulkActionResponse>,
}

impl BulkResponseItem {
  pub fn action_response(&self) -> Option<&BulkActionResponse> {
    self
      .index
      .as_ref()
      .or_else(|| self.update.as_ref())
      .or_else(|| self.delete.as_ref())
  }

  pub fn is_success(&self) -> bool {
    self
      .action_response()
      .map(|response| response.status < 300)
      .unwrap_or(false)
  }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulkResponse {
  pub errors: bool,
  pub items: Vec<BulkResponseItem>,
}

#[derive(Clone, Debug, Default)]
pub struct BulkResult {
  pub successes: Vec<String>,
  pub failures: Vec<String>,
}

impl BulkResult {
  pub fn from_response(response: BulkResponse) -> BulkResult {
    let mut result: BulkResult = Default::default();

    for item in response.items {
      match item.action_response() {
        Some(action_response) if action_response.status < 300 => result.successes.push(action_response.id.to_owned()),
        Some(action_response) => result.failures.push(action_response.id.to_owned()),
        None => (),
      }
    }

    result
  }
}

pub enum BulkAction<T> {
  Index(String, T),
  Upsert(String, T),
  Delete(String),
}

impl<T> BulkAction<T>
where
  T: serde::Serialize,
{
  pub fn index(id: String, doc: T) -> BulkAction<T> {
    BulkAction::Index(id, doc)
  }

  pub fn upsert(id: String, doc: T) -> BulkAction<T> {
    BulkAction::Upsert(id, doc)
  }

  pub fn to_bytes(&self) -> Result<Bytes, Problem> {
    match self {
      BulkAction::Index(id, doc) => {
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        to_writer(&mut buf, &json!(  {"index": { "_id": id, "_type": "_doc" } }))?;
        buf.write_all(b"\n")?;
        to_writer(&mut buf, &doc)?;
        buf.write_all(b"\n")?;
        Ok(buf.into())
      }
      BulkAction::Upsert(id, doc) => {
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        to_writer(&mut buf, &json!(  {"update": { "_id": id, "_type": "_doc" } }))?;
        buf.write_all(b"\n{\"doc\":")?;
        to_writer(&mut buf, &doc)?;
        buf.write_all(b",\"doc_as_upsert\":true}\n")?;
        Ok(buf.into())
      }
      BulkAction::Delete(id) => {
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        to_writer(&mut buf, &json!(  {"delete": { "_id": id, "_type": "_doc" } }))?;
        buf.write_all(b"\n")?;
        Ok(buf.into())
      }
    }
  }
}

pub struct BulkActions<B, T>(pub B)
where
  B: IntoIterator<Item = BulkAction<T>>;

impl<B, T, I> IntoClientRequest for BulkActions<B, T>
where
  B: IntoIterator<Item = BulkAction<T>, IntoIter = I> + 'static,
  I: Iterator<Item = BulkAction<T>> + Send + Sync + 'static,
  T: serde::Serialize,
{
  fn apply_body(self, request: RequestBuilder) -> RequestBuilder {
    let body = Body::wrap_stream(stream::iter(self.0.into_iter().map(|a| a.to_bytes())));
    request.header(header::CONTENT_TYPE, "application/x-ndjson").body(body)
  }
}

pub struct ElasticsearchUrlBuilder {
  elasticsearch_base_url: String,
  index_name: String,
}

impl ElasticsearchUrlBuilder {
  pub fn new(elasticsearch_base_url: String, index_name: String) -> ElasticsearchUrlBuilder {
    ElasticsearchUrlBuilder {
      elasticsearch_base_url,
      index_name,
    }
  }

  pub fn index(&self) -> String {
    format!("{}/{}", self.elasticsearch_base_url, self.index_name)
  }

  pub fn mapping(&self) -> String {
    format!("{}/_mapping/_doc", self.index())
  }

  pub fn search(&self) -> String {
    format!("{}/_search", self.index())
  }

  pub fn mget(&self) -> String {
    format!("{}/_doc/_mget", self.index())
  }

  pub fn create(&self, id: String) -> String {
    format!(
      "{}/_doc/{}/_create?refresh=true",
      self.index(),
      encode_url_component(id),
    )
  }

  pub fn update(&self, id: String) -> String {
    format!("{}/_doc/{}/_update", self.index(), encode_url_component(id),)
  }
  pub fn delete(&self, id: String) -> String {
    format!("{}/_doc/{}?refresh=wait_for", self.index(), encode_url_component(id),)
  }

  pub fn delete_by_query(&self) -> String {
    format!("{}/_delete_by_query", self.index())
  }
}
