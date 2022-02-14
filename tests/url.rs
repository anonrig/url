use url_wasm::url::URL;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn should_parse_protocol() {
    let url = URL::new(Some("http://example\t.\norg".to_string()), None);
    assert_eq!(url.get_protocol(), "http:".to_string());
}

#[wasm_bindgen_test]
fn should_set_protocol() {
    let mut url = URL::new(Some("https://www.google.com".to_string()), None);
    assert_eq!(url.get_protocol(), "https:".to_string());
    url.set_protocol("http:".to_string());
    assert_eq!(url.get_protocol(), "http:".to_string());
    url.set_protocol("https".to_string()); // testing without ':' at the end
    assert_eq!(url.get_protocol(), "http:".to_string());
    url.set_protocol(":".to_string());
    assert_eq!(url.get_protocol(), "http:".to_string());
}
