use serde_derive::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Status {
  version: String,
}

impl Status {
  pub fn new<S: ToString>(version: Option<S>) -> Status {
    Status {
      version: version.map(|s| s.to_string()).unwrap_or_else(|| "UNKNOWN".to_string()),
    }
  }
}
