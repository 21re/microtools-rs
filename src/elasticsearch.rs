use super::serde_field_value;
use super::{IntoClientRequest, Problem};
use actix_web::client::{ClientRequest, ClientRequestBuilder};
use bytes::Bytes;
use futures::stream;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use serde_json::to_vec;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
  Bool(bool),
  Int(i64),
  String(String),
}

impl Value {
  pub fn to_string(&self) -> String {
    match self {
      Value::Bool(b) => format!("{}", b),
      Value::Int(i) => format!("{}", i),
      Value::String(str) => str.clone(),
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

impl Serialize for Sort {
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

impl<'de> Deserialize<'de> for Sort {
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
}

#[derive(Clone, Debug, Deserialize)]
pub struct QueryHits<I, T> {
  pub total: u64,
  pub hits: Vec<Doc<I, T>>,
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
  id: String,
  status: u16,
  result: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulkResponseItem {
  index: Option<BulkActionResponse>,
  update: Option<BulkActionResponse>,
  delete: Option<BulkActionResponse>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BulkResponse {
  errors: bool,
  items: Vec<BulkResponseItem>,
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
      match item.index.or(item.update).or(item.delete) {
        Some(ref action_response) if action_response.status < 300 => {
          result.successes.push(action_response.id.to_owned())
        }
        Some(action_response) => result.failures.push(action_response.id),
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
  T: Serialize,
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
        let mut result = Bytes::new();
        result.extend_from_slice(&to_vec(&json!(  {"index": { "_id": id, "_type": "_doc" } }))?);
        result.extend_from_slice(b"\n");
        result.extend_from_slice(&to_vec(doc)?);
        result.extend_from_slice(b"\n");
        Ok(result)
      }
      BulkAction::Upsert(id, doc) => {
        let mut result = Bytes::new();
        result.extend_from_slice(&to_vec(&json!(  {"update": { "_id": id, "_type": "_doc" } }))?);
        result.extend_from_slice(b"\n");
        result.extend_from_slice(&to_vec(&json!({"doc": doc, "doc_as_upsert": true}))?);
        result.extend_from_slice(b"\n");
        Ok(result)
      }
      BulkAction::Delete(id) => {
        let mut result = Bytes::new();
        result.extend_from_slice(&to_vec(&json!(  {"delete": { "_id": id, "_type": "_doc" } }))?);
        result.extend_from_slice(b"\n");
        Ok(result)
      }
    }
  }
}

pub struct BulkActions<B, T>(pub B)
where
  B: IntoIterator<Item = BulkAction<T>>;

impl<B, T> IntoClientRequest for BulkActions<B, T>
where
  B: IntoIterator<Item = BulkAction<T>> + 'static,
  T: Serialize,
{
  fn apply_body(self, builder: &mut ClientRequestBuilder) -> Result<ClientRequest, Problem> {
    builder
      .content_type("application/x-ndjson")
      .streaming(stream::iter_result(self.0.into_iter().map(|a| a.to_bytes())))
      .map_err(Problem::from)
  }
}
