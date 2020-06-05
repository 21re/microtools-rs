use crate::business_result::BusinessResult;
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
  #[serde(rename = "keyId")]
  pub key_id: String,
  #[serde(rename = "keyType")]
  pub key_type: String,
  pub raw: String,
  pub crypted: String,
  pub expires: u64,
  pub claims: Map<String, Value>,
}

pub struct TokenCreator {
  claims: Map<String, Value>,
  current: Mutex<Option<Token>>,
  client: Client,
}

impl TokenCreator {
  pub fn for_service(service_name: &str, scopes: &[(&str, &[&str])]) -> TokenCreator {
    let mut claims: Map<String, Value> = Map::new();

    claims.insert("sub".to_string(), Value::String(format!("service/{}", service_name)));

    let mut scopes_map: Map<String, Value> = Map::new();

    for (key, values_raw) in scopes {
      let values = values_raw.iter().map(|v| Value::String(v.to_string())).collect();
      scopes_map.insert(key.to_string(), Value::Array(values));
    }

    claims.insert("scopes".to_string(), Value::Object(scopes_map));

    TokenCreator {
      claims,
      current: Mutex::new(None),
      client: Client::new(),
    }
  }

  pub async fn get_token(&mut self) -> BusinessResult<Token> {
    let now_plus_grace = SystemTime::now() + Duration::from_secs(60);
    let unixtime = now_plus_grace.duration_since(UNIX_EPOCH)?.as_secs();
    let current = self.current.get_mut()?;

    match current.as_ref() {
      Some(token) if token.expires > unixtime => Ok(token.clone()),
      _ => {
        let response = self
        .client
        .post("http://localhost:12345/v1/tokens")
        .timeout(Duration::from_secs(30))
        .json(&self.claims)
        .send()
        .await?;
        let next_token = response.json::<Token>().await?;

        current.replace(next_token.clone());

        Ok(next_token)  
      }
    }
  }
}
