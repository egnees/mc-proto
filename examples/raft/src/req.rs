use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub enum Request {
    Init { nodes: usize, me: usize },
}

impl Into<String> for Request {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl From<String> for Request {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap()
    }
}
