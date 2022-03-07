use js_sys::Array;
use std::collections::HashMap;
use std::vec::Vec;
use wasm_bindgen::prelude::*;

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
pub struct URLSearchParams {
    params: Vec<(String, String)>,
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
impl URLSearchParams {
    /// Create a new URLSearchParams instance
    #[wasm_bindgen(constructor)]
    pub fn new(props: &JsValue) -> Result<URLSearchParams, JsValue> {
        let mut internal_params: Vec<(String, String)> = Vec::new();

        if props.is_null() || props.is_undefined() {
            // do nothing
        } else if Array::is_array(props) {
            let iterator = js_sys::try_iter(props)?.unwrap();

            for i in iterator {
                let key_value_array: Vec<String> = serde_wasm_bindgen::from_value(i.unwrap())?;

                if key_value_array.len() >= 2 {
                    let mut e = key_value_array.iter();

                    if let (Some(key), Some(value)) = (e.next(), e.next()) {
                        internal_params.push((key.clone(), value.clone()));
                    }
                }
            }
        } else if props.is_object() {
            let hash: HashMap<String, String> =
                serde_wasm_bindgen::from_value(JsValue::from(props))?;

            internal_params = hash.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>()
        } else if props.is_string() {
            if let Some(mut value) = props.as_string() {
                value = if value.starts_with('?') {
                    value[1..value.len()].to_string()
                } else {
                    value
                };
                value.split('&').for_each(|key_value| {
                    let mut pair = key_value.split('=');

                    if let (Some(key), Some(value)) = (pair.next(), pair.next()) {
                        internal_params.push((key.to_string(), value.to_string()));
                    }
                })
            }
        }

        Ok(URLSearchParams {
            params: internal_params,
        })
    }

    /// Appends a specified key-value pair as a new search parameter.
    #[wasm_bindgen]
    pub fn append(&mut self, name: String, value: String) {
        self.params.push((name, value))
    }

    /// Returns an iterator allowing iteration through all key/value pairs contained in this object.
    /// The iterator returns key/value pairs in the same order as they appear in the query string.
    #[wasm_bindgen]
    pub fn entries(&self) -> Array {
        self.params
            .iter()
            .map(|p| {
                let as_array = Array::of2(
                    &JsValue::from_str(p.0.as_str()),
                    &JsValue::from_str(p.1.as_str()),
                );
                as_array
            })
            .collect::<js_sys::Array>()
    }

    /// Deletes the given search parameter and all its associated values from the list
    /// of all search parameters.
    #[wasm_bindgen]
    pub fn delete(&mut self, name: String) {
        self.params.retain(|p| p.0 != name);
    }

    /// Allows iteration through all values contained in this object via a callback function.
    #[wasm_bindgen(js_name = forEach)]
    pub fn for_each(&self, callback: &js_sys::Function) {
        let null = JsValue::null();

        for parameter in &self.params {
            let _ = callback.call2(
                &null,
                &JsValue::from_str(&parameter.1),
                &JsValue::from_str(&parameter.0),
            );
        }
    }

    /// Returns a string if the given search parameter is found; otherwise `null`.
    #[wasm_bindgen]
    pub fn get(&self, name: String) -> Option<String> {
        self.params
            .iter()
            .find(|p| p.0 == name)
            .map(|p| p.1.clone())
    }

    /// Returns all the values associated with a given search parameter as an array.
    #[wasm_bindgen(js_name = getAll)]
    pub fn get_all(&self, name: String) -> Array {
        self.params
            .iter()
            .filter_map(|p| {
                if p.0 == name {
                    Some(JsValue::from(&p.1))
                } else {
                    None
                }
            })
            .collect::<Array>()
    }

    /// Returns a boolean value that indicates whether a parameter with the specified name exists.
    #[wasm_bindgen]
    pub fn has(&self, name: String) -> bool {
        self.params.iter().any(|p| p.0 == name)
    }

    /// Returns an iterator allowing iteration through all keys contained in this object. The keys are USVString objects.
    #[wasm_bindgen]
    pub fn keys(&self) -> Array {
        self.params
            .iter()
            .map(|p| JsValue::from(&p.0))
            .collect::<Array>()
    }

    /// Sets the value associated with a given search parameter to the given value.
    /// If there were several matching values, this method deletes the others.
    /// If the search parameter doesn't exist, this method creates it.
    #[wasm_bindgen]
    pub fn set(&mut self, name: String, value: String) {
        self.params.retain(|p| p.0 != name);
        self.params.push((name, value));
    }

    /// Sorts all key/value pairs contained in this object in place and returns undefined.
    /// The sort order is according to unicode code points of the keys.
    #[wasm_bindgen]
    pub fn sort(&mut self) {
        self.params.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0))
    }

    /// Returns a query string suitable for use in a URL.
    #[wasm_bindgen(js_name = toString)]
    pub fn to_js_string(&self) -> String {
        self.params
            .iter()
            .map(|p| format!("{}={}", p.0, p.1))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Returns an iterator allowing iteration through all values contained in this object.
    #[wasm_bindgen]
    pub fn values(&self) -> Array {
        self.params
            .iter()
            .map(|p| JsValue::from(&p.1))
            .collect::<Array>()
    }
}
