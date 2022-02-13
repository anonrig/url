use url_wasm::url::URL;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn should_parse_protocol() {
    let url = URL::new(Some("http://example\t.\norg".to_string()), None);
    assert_eq!(url.get_protocol(), "http:".to_string());
}
