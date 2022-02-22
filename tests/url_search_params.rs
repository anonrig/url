use js_sys::{Array, Object, Reflect};
use url_wasm::url_search_params::URLSearchParams;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

fn create_array_from_tuples(input: Vec<(String, String)>) -> Array {
    let array = Array::new();

    for (key, value) in input {
        let inner_array = Array::new();
        inner_array.push(&key.into());
        inner_array.push(&value.into());
        array.push(&inner_array);
    }

    array
}

#[wasm_bindgen_test]
fn new_should_accept_undefined() {
    let empty_params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    assert_eq!(empty_params.to_js_string(), "".to_string());
}

#[wasm_bindgen_test]
fn new_should_accept_object() {
    let props = Object::new();
    let _ = Reflect::set(&props, &"name".into(), &"value".into());
    let mut params = URLSearchParams::new(&JsValue::from(props)).unwrap();
    params.set("hello".to_string(), "world".to_string());
    assert_eq!(params.to_js_string(), "name=value&hello=world".to_string());
}

#[wasm_bindgen_test]
fn new_should_accept_array() {
    let tuples = vec![
        ("name".to_string(), "value".to_string()),
        ("name".to_string(), "second-value".to_string()),
    ];
    let props = create_array_from_tuples(tuples);
    let params = URLSearchParams::new(&JsValue::from(props)).unwrap();
    assert_eq!(
        params.to_js_string(),
        "name=value&name=second-value".to_string()
    );
}

#[wasm_bindgen_test]
fn new_should_accept_string() {
    let params = URLSearchParams::new(&JsValue::from("name=url&type=parser".to_string())).unwrap();
    assert_eq!(
        params.values().to_vec(),
        ["url".to_string(), "parser".to_string()].map(JsValue::from)
    );
    assert_eq!(
        params.keys().to_vec(),
        ["type".to_string(), "name".to_string()].map(JsValue::from)
    );
}

#[wasm_bindgen_test]
fn new_should_accept_strings_starting_with_questionmark() {
    let params = URLSearchParams::new(&JsValue::from("?name=url".to_string())).unwrap();
    assert_eq!(
        params.values().to_vec(),
        ["url".to_string()].map(JsValue::from)
    );
    assert_eq!(
        params.keys().to_vec(),
        ["name".to_string()].map(JsValue::from)
    );
}

#[wasm_bindgen_test]
fn has_returns_correct() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("name".to_string(), "value".to_string());
    assert_eq!(params.has("name".to_string()), true);
    assert_eq!(params.has("unknown".to_string()), false);
}

#[wasm_bindgen_test]
fn append_should_care_about_order() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.append("name".to_string(), "value1".to_string());
    params.append("name".to_string(), "value2".to_string());
    assert_eq!(
        params.get("name".to_string()).unwrap(),
        "value1".to_string()
    );
    assert_eq!(params.to_js_string(), "name=value1&name=value2".to_string());
}

#[wasm_bindgen_test]
fn delete_should_literally_delete() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("hello".to_string(), "world".to_string());
    assert_eq!(params.to_js_string(), "hello=world".to_string());
    params.delete("hello".to_string());
    assert_eq!(params.to_js_string(), "".to_string());
}

#[wasm_bindgen_test]
fn get_should_return_value() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("hello".to_string(), "world".to_string());
    assert_eq!(
        params.get("hello".to_string()).unwrap(),
        "world".to_string()
    );
    assert_eq!(params.get("unknown".to_string()), None);
}

#[wasm_bindgen_test]
fn get_all_should_filter_by_name() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("name".to_string(), "first-value".to_string());
    params.append("name".to_string(), "second-value".to_string());

    assert_eq!(
        params.get_all("name".to_string()).to_vec(),
        ["first-value".to_string(), "second-value".to_string()]
    );
}

#[wasm_bindgen_test]
fn get_all_returns_empty_array() {
    let params = URLSearchParams::new(&JsValue::undefined()).unwrap();

    assert_eq!(params.get_all("unknown".to_string()).to_vec().len(), 0)
}

#[wasm_bindgen_test]
fn has_should_return_boolean() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("name".to_string(), "value".to_string());
    assert_eq!(params.has("name".to_string()), true);
    assert_eq!(params.has("unknown".to_string()), false);
}

#[wasm_bindgen_test]
fn keys_should_return_all_unique_keys() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.append("common".to_string(), "value1".to_string());
    params.append("common".to_string(), "value2".to_string());
    params.append("common".to_string(), "value3".to_string());
    params.set("hello".to_string(), "world".to_string());
    params.set("abc".to_string(), "first".to_string());
    assert_eq!(
        params.keys().to_vec(),
        ["abc".to_string(), "hello".to_string(), "common".to_string()].map(JsValue::from)
    );
}

#[wasm_bindgen_test]
fn set_should_update_and_override() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("name".to_string(), "value".to_string());
    assert_eq!(params.has("name".to_string()), true);
    params.append("mutate".to_string(), "value".to_string());
    params.append("mutate".to_string(), "value2".to_string());
    assert_eq!(
        params.to_js_string(),
        "name=value&mutate=value&mutate=value2".to_string()
    );
    params.set("mutate".to_string(), "overriden".to_string());
    assert_eq!(
        params.to_js_string(),
        "name=value&mutate=overriden".to_string()
    );
}

#[wasm_bindgen_test]
fn sort_should_update_internal_vector() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    params.set("a".to_string(), "a_value".to_string());
    params.set("c".to_string(), "c_value".to_string());
    params.set("b".to_string(), "b_value".to_string());
    assert_eq!(params.to_js_string(), "a=a_value&c=c_value&b=b_value");
    params.sort();
    assert_eq!(params.to_js_string(), "a=a_value&b=b_value&c=c_value");
}

#[wasm_bindgen_test]
fn to_js_string_should_return_string() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    assert_eq!(params.to_js_string(), "".to_string());
    params.set("hello".to_string(), "world".to_string());
    assert_eq!(params.to_js_string(), "hello=world".to_string());
}

#[wasm_bindgen_test]
fn values_should_return_only_values() {
    let mut params = URLSearchParams::new(&JsValue::undefined()).unwrap();
    assert_eq!(params.values().to_vec().len(), 0);
    params.set("name".to_string(), "value".to_string());
    params.set("second".to_string(), "second-value".to_string());
    assert_eq!(
        params.values().to_vec(),
        ["value", "second-value"].map(JsValue::from)
    );
}
