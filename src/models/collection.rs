use crate::models::Request;

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub requests: Vec<Request>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            name,
            requests: Vec::new(),
        }
    }
}
