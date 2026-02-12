pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}
