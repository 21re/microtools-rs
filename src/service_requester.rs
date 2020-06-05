use url::form_urlencoded::byte_serialize;

pub fn encode_url_component<S: AsRef<[u8]>>(value: S) -> String {
  byte_serialize(value.as_ref()).collect::<String>()
}
