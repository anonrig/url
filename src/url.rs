#[derive(Clone, Debug)]
pub struct URL {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: Option<String>,
    pub port: Option<String>,
    pub path: Vec<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl Default for URL {
    fn default() -> Self {
        URL::new()
    }
}

impl URL {
    pub fn new() -> Self {
        URL {
            scheme: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            host: None,
            port: None,
            path: vec![],
            query: None,
            fragment: None,
        }
    }
}
