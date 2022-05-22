pub struct URL {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub path: Vec<String>,
    pub query: Option<String>,
    pub fragment: Option<String>
}

impl URL {
    pub fn new() -> URL {
        URL {
            scheme: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            host: None,
            port: None,
            path: vec![],
            query: None,
            fragment: None
        }
    }
}
