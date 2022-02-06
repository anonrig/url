use wasm_bindgen::prelude::*;
use std::vec::Vec;
use js_sys::{Array};

struct Parameter {
  name: String,
  value: String,
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
pub struct URLSearchParams {
  params: Vec<Parameter>
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
impl URLSearchParams {
  #[wasm_bindgen(constructor)]
  pub fn new(_params: Option<String>) -> URLSearchParams {
    URLSearchParams {
      params: Vec::new()
    }
  }

  /// Appends a specified key-value pair as a new search parameter.
  #[wasm_bindgen]
  pub fn append(&mut self, name: String, value: String) {
    self.params.push(Parameter { name, value })
  }

  /// Deletes the given search parameter and all its associated values from the list
  /// of all search parameters.
  #[wasm_bindgen]
  pub fn delete(&mut self, name: String) {
    self.params.retain(|pair| pair.name != name);
  }

  /// Returns a string if the given search parameter is found; otherwise `null`.
  #[wasm_bindgen]
  pub fn get(&self, name: String) -> Option<String> {
    self.params.iter().find(|p| p.name == name).map(|p| p.value.clone())
  }

  /// Returns all the values associated with a given search parameter as an array.
  #[wasm_bindgen(js_name = getAll)]
  pub fn get_all(&self, name: String) -> JsValue {
    return JsValue::from(
      self.params.iter()
        .filter(|p| p.name == name)
        .map(|p| JsValue::from_str(&p.value))
        .collect::<Array>()
    );
  }

  /// Returns a boolean value that indicates whether a parameter with the specified name exists.
  #[wasm_bindgen]
  pub fn has(&self, name: String) -> bool {
    self.params.iter().any(|p| p.name == name)
  }

  /// Sets the value associated with a given search parameter to the given value.
  /// If there were several matching values, this method deletes the others.
  /// If the search parameter doesn't exist, this method creates it.
  #[wasm_bindgen]
  pub fn set(&mut self, name: String, value: String) {
    self.params.retain(|p| p.name != name);
    self.append(name, value);
  }

  /// Returns a query string suitable for use in a URL.
  #[wasm_bindgen(js_name = toString)]
  pub fn to_js_string(&self) -> String {
    self.params
      .iter()
      .map(|p| {
        format!("{}={}", p.name, p.value)
      })
      .collect::<Vec<_>>().join("&")
  }
}
