use wasm_bindgen::prelude::*;

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
pub struct URL {
    hash: String,
    host: String,
    hostname: String,
    href: String,
    origin: String,
    password: String,
    pathname: String,
    port: String,
    protocol: String,
    search: String,
}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
impl URL {
    #[wasm_bindgen(constructor)]
    pub fn new(_url: String, _base: Option<String>) -> URL {
        URL {
            hash: "".to_string(),
            host: "".to_string(),
            hostname: "".to_string(),
            href: "".to_string(),
            origin: "".to_string(),
            password: "".to_string(),
            pathname: "".to_string(),
            port: "".to_string(),
            protocol: "".to_string(),
            search: "".to_string(),
        }
    }

    #[wasm_bindgen(getter = hash)]
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    #[wasm_bindgen(getter = host)]
    pub fn get_host(&self) -> String {
        self.host.clone()
    }

    #[wasm_bindgen(getter = hostname)]
    pub fn get_hostname(&self) -> String {
        self.hostname.clone()
    }

    #[wasm_bindgen(getter = href)]
    pub fn get_href(&self) -> String {
        self.href.clone()
    }

    #[wasm_bindgen(getter = origin)]
    pub fn get_origin(&self) -> String {
        self.origin.clone()
    }

    #[wasm_bindgen(getter = password)]
    pub fn get_password(&self) -> String {
        self.password.clone()
    }

    #[wasm_bindgen(getter = pathname)]
    pub fn get_pathname(&self) -> String {
        self.pathname.clone()
    }

    #[wasm_bindgen(getter = port)]
    pub fn get_port(&self) -> String {
        self.port.clone()
    }

    #[wasm_bindgen(getter = protocol)]
    pub fn get_protocol(&self) -> String {
        self.protocol.clone()
    }

    #[wasm_bindgen(getter = search)]
    pub fn get_search(&self) -> String {
        self.search.clone()
    }
}
