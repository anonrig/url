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
    assert_eq!(
        params.get("name".to_string()).unwrap(),
        "value1".to_string()
    );
}

#[test]
fn sort_should_update() {
    let mut params = URLSearchParams::new(None);
    params.set("a".to_string(), "a_value".to_string());
    params.set("c".to_string(), "c_value".to_string());
    params.set("b".to_string(), "b_value".to_string());
    assert_eq!(params.to_js_string(), "a=a_value&c=c_value&b=b_value");
    params.sort();
    assert_eq!(params.to_js_string(), "a=a_value&b=b_value&c=c_value");
}
