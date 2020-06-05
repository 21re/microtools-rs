use crate::business_result::BusinessResult;
use crate::problem::Problem;
use crate::ws_try::SendClientRequestExt;
use actix::{Actor, ActorFuture, ActorResponse, Context, Handler, Message, WrapFuture};
use actix_web::client::Client;
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
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

#[derive(Message)]
#[rtype(result = "BusinessResult<Token>")]
struct GetToken;

pub struct TokenCreator {
  claims: Map<String, Value>,
  current: Option<Token>,
  client: Client,
}

pub async fn get_token(actor: &actix::Addr<TokenCreator>) -> BusinessResult<Token> {
  actor.send(GetToken).await?
}

impl TokenCreator {
  pub fn for_service(service_name: &str, scopes: &[(&str, &[&str])]) -> TokenCreator {
    let mut claims: Map<String, Value> = Map::new();

    claims.insert("sub".to_string(), Value::String(format!("service/{}", service_name)));

    let mut scopes_map: Map<String, Value> = Map::new();

    for (key, values_raw) in scopes {
      let values = values_raw.iter().map(|v| Value::String((*v).to_string())).collect();
      scopes_map.insert((*key).to_string(), Value::Array(values));
    }

    claims.insert("scopes".to_string(), Value::Object(scopes_map));

    TokenCreator {
      claims,
      current: None,
      client: Client::new(),
    }
  }
}

impl Handler<GetToken> for TokenCreator {
  type Result = ActorResponse<Self, Token, Problem>;

  fn handle(&mut self, _msg: GetToken, _ctx: &mut actix::Context<TokenCreator>) -> Self::Result {
    let now_plus_grace = SystemTime::now() + Duration::from_secs(60);
    let unixtime = match now_plus_grace.duration_since(UNIX_EPOCH) {
      Ok(duration) => duration.as_secs(),
      Err(error) => return ActorResponse::reply(Err(Problem::from(error))),
    };
    match self.current {
      Some(ref token) if token.expires > unixtime => ActorResponse::reply(Ok(token.clone())),
      _ => {
        let token_response = self
          .client
          .post("http://localhost:12345/v1/tokens")
          .timeout(Duration::from_secs(30))
          .send_json(&self.claims)
          .expect_success::<Token>();

        ActorResponse::r#async(token_response.into_actor(self).map(|maybe_token, actor, _| {
          if let Ok(ref token) = maybe_token {
            actor.current = Some(token.clone());
          }
          maybe_token
        }))
      }
    }
  }
}

impl Actor for TokenCreator {
  type Context = Context<Self>;
}
