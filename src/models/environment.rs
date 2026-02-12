use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

impl Environment {
    pub fn new(name: String) -> Self {
        Self {
            name,
            variables: HashMap::new(),
        }
    }
}
