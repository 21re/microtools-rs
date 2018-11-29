use service_requester::encode_url_component;
use spectral::prelude::*;

#[test]
fn test_encode_urlcomponent() {
  assert_that(&encode_url_component("abcdef").as_str()).is_equal_to("abcdef");
  assert_that(&encode_url_component("abc de f").as_str()).is_equal_to("abc+de+f");
  assert_that(&encode_url_component("abc de f\\/?&").as_str()).is_equal_to("abc+de+f%5C%2F%3F%26");
  assert_that(&encode_url_component("äbcdü").as_str()).is_equal_to("%C3%A4bcd%C3%BC");
}
