use url::url_search_params::URLSearchParams;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn has_returns_correct() {
    let mut params = URLSearchParams::new(None);
    params.set("name".to_string(), "value".to_string());
    assert_eq!(params.has("name".to_string()), true);
    assert_eq!(params.has("unknown".to_string()), false);
}

#[wasm_bindgen_test]
fn append_should_care_about_order() {
    let mut params = URLSearchParams::new(None);
    params.append("name".to_string(), "value1".to_string());
    params.append("name".to_string(), "value2".to_string());
    assert_eq!(
        params.get("name".to_string()).unwrap(),
        "value1".to_string()
    );
}

#[wasm_bindgen_test]
fn sort_should_update() {
    let mut params = URLSearchParams::new(None);
    params.set("a".to_string(), "a_value".to_string());
    params.set("c".to_string(), "c_value".to_string());
    params.set("b".to_string(), "b_value".to_string());
    assert_eq!(params.to_js_string(), "a=a_value&c=c_value&b=b_value");
    params.sort();
    assert_eq!(params.to_js_string(), "a=a_value&b=b_value&c=c_value");
}

#[wasm_bindgen_test]
fn keys_should_return_an_iterator() {
    let mut params = URLSearchParams::new(None);
    params.set("name".to_string(), "value".to_string());
    params.set("version".to_string(), "1.0.0".to_string());
    assert_eq!(
        params.keys().to_vec(),
        ["name".to_string(), "version".to_string()]
    )
}

#[wasm_bindgen_test]
fn get_all_should_filter_by_name() {
    let mut params = URLSearchParams::new(None);
    params.set("name".to_string(), "first-value".to_string());
    params.append("name".to_string(), "second-value".to_string());

    assert_eq!(
        params.get_all("name".to_string()).to_vec(),
        ["first-value".to_string(), "second-value".to_string()]
    );
}

#[wasm_bindgen_test]
fn get_all_returns_empty_array() {
    let params = URLSearchParams::new(None);

    assert_eq!(params.get_all("unknown".to_string()).to_vec().len(), 0)
}
