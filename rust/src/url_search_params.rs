use js_sys::Array;
use std::vec::Vec;
use wasm_bindgen::prelude::*;

struct Parameter {
    name: String,
    value: String,
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
pub struct URLSearchParams {
    params: Vec<Parameter>,
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
impl URLSearchParams {
    /// Create a new URLSearchParams instance
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// assert_eq!(search_params.to_js_string(), "".to_string());
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(_params: Option<String>) -> URLSearchParams {
        URLSearchParams { params: Vec::new() }
    }

    /// Appends a specified key-value pair as a new search parameter.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.append("name".to_string(), "value1".to_string());
    /// search_params.append("name".to_string(), "value2".to_string());
    /// assert_eq!(search_params.to_js_string(), "name=value1&name=value2".to_string());
    /// ```
    #[wasm_bindgen]
    pub fn append(&mut self, name: String, value: String) {
        self.params.push(Parameter { name, value })
    }

    /// Deletes the given search parameter and all its associated values from the list
    /// of all search parameters.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("name".to_string(), "value".to_string());
    /// assert_eq!(search_params.has("name".to_string()), true);
    /// search_params.delete("name".to_string());
    /// assert_eq!(search_params.has("name".to_string()), false);
    /// ```
    #[wasm_bindgen]
    pub fn delete(&mut self, name: String) {
        self.params.retain(|pair| pair.name != name);
    }

    /// Returns a string if the given search parameter is found; otherwise `null`.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("name".to_string(), "value".to_string());
    /// assert_eq!(search_params.get("name".to_string()).unwrap(), "value".to_string());
    /// ```
    #[wasm_bindgen]
    pub fn get(&self, name: String) -> Option<String> {
        self.params
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value.clone())
    }

    /// Returns all the values associated with a given search parameter as an array.
    #[wasm_bindgen(js_name = getAll)]
    pub fn get_all(&self, name: String) -> JsValue {
        return JsValue::from(
            self.params
                .iter()
                .filter(|p| p.name == name)
                .map(|p| JsValue::from_str(&p.value))
                .collect::<Array>(),
        );
    }

    /// Returns a boolean value that indicates whether a parameter with the specified name exists.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("name".to_string(), "value".to_string());
    /// assert_eq!(search_params.has("name".to_string()), true);
    /// assert_eq!(search_params.has("unknown".to_string()), false);
    /// ```
    #[wasm_bindgen]
    pub fn has(&self, name: String) -> bool {
        self.params.iter().any(|p| p.name == name)
    }

    /// Sets the value associated with a given search parameter to the given value.
    /// If there were several matching values, this method deletes the others.
    /// If the search parameter doesn't exist, this method creates it.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("name".to_string(), "value".to_string());
    /// assert_eq!(search_params.get("name".to_string()).unwrap(), "value".to_string())
    /// ```
    #[wasm_bindgen]
    pub fn set(&mut self, name: String, value: String) {
        self.params.retain(|p| p.name != name);
        self.append(name, value);
    }

    /// Sorts all key/value pairs contained in this object in place and returns undefined.
    /// The sort order is according to unicode code points of the keys.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("a".to_string(), "a_value".to_string());
    /// search_params.set("c".to_string(), "c_value".to_string());
    /// search_params.set("b".to_string(), "b_value".to_string());
    /// search_params.sort();
    /// assert_eq!(search_params.to_js_string(), "a=a_value&b=b_value&c=c_value");
    /// ```
    #[wasm_bindgen]
    pub fn sort(&mut self) {
        self.params.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name))
    }

    /// Returns a query string suitable for use in a URL.
    ///
    /// ```
    /// use url::url_search_params::URLSearchParams;
    ///
    /// let mut search_params = URLSearchParams::new(None);
    /// search_params.set("name".to_string(), "value".to_string());
    /// assert_eq!(search_params.to_js_string(), "name=value");
    /// ```
    #[wasm_bindgen(js_name = toString)]
    pub fn to_js_string(&self) -> String {
        self.params
            .iter()
            .map(|p| format!("{}={}", p.name, p.value))
            .collect::<Vec<_>>()
            .join("&")
    }
}
