use crate::business_result::BusinessResult;
use crate::problem::Problem;
use crate::ws_try;
use actix::{Actor, ActorFuture, ActorResponse, Context, Handler, Message, WrapFuture};
use actix_web::client;
use futures::Future;
use log::error;
use serde_json::{Map, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde_derive::{Serialize, Deserialize};

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
}

pub fn get_token(actor: &actix::Addr<TokenCreator>) -> impl Future<Item = Token, Error = Problem> {
  actor.send(GetToken).flatten()
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

    TokenCreator { claims, current: None }
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
        let token_request = match client::post("http://localhost:12345/v1/tokens")
          .timeout(Duration::from_secs(30))
          .json(&self.claims)
        {
          Ok(request) => request,
          Err(error) => {
            error!("Token request failed: {}", error);
            return ActorResponse::reply(Err(Problem::from(error)));
          }
        };

        ActorResponse::r#async(ws_try::expect_success(token_request).into_actor(self).map(
          |token: Token, actor: &mut TokenCreator, _| {
            actor.current = Some(token.clone());
            token
          },
        ))
      }
    }
  }
}

impl Actor for TokenCreator {
  type Context = Context<Self>;
}
