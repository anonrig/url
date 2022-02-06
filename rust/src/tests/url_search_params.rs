use crate::url_search_params::URLSearchParams;

#[test]
fn has_returns_correct() {
  let mut params = URLSearchParams::new(None);
  params.set("name".to_string(), "value".to_string());
  assert_eq!(params.has("name".to_string()), true);
  assert_eq!(params.has("unknown".to_string()), false);
}

#[test]
fn append_should_care_about_order() {
  let mut params = URLSearchParams::new(None);
  params.append("name".to_string(), "value1".to_string());
  params.append("name".to_string(), "value2".to_string());
  assert_eq!(params.get("name".to_string()).unwrap(), "value1".to_string());
}
