use serde::de::Visitor;
use serde::de::{Error, MapAccess};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

struct TupleMapVisitor<K, V> {
  marker: PhantomData<(K, V)>,
}

impl<K, V> TupleMapVisitor<K, V> {
  pub fn new() -> Self {
    TupleMapVisitor { marker: PhantomData }
  }
}

impl<'de, K, V> Visitor<'de> for TupleMapVisitor<K, V>
where
  K: Deserialize<'de>,
  V: Deserialize<'de>,
{
  type Value = (K, V);

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a map with one key,value")
  }

  #[inline]
  fn visit_map<T>(self, mut access: T) -> Result<(K, V), T::Error>
  where
    T: MapAccess<'de>,
  {
    if let Some((key, value)) = access.next_entry()? {
      Ok((key, value))
    } else {
      Err(Error::invalid_length(0, &self))
    }
  }
}

pub fn serialize<K, V, S>(key: &K, value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  K: Serialize,
  V: Serialize,
{
  let mut map = serializer.serialize_map(Some(1))?;
  map.serialize_entry(key, value)?;
  map.end()
}

pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<(K, V), D::Error>
where
  D: Deserializer<'de>,
  K: Deserialize<'de>,
  V: Deserialize<'de>,
{
  deserializer.deserialize_map(TupleMapVisitor::new())
}
