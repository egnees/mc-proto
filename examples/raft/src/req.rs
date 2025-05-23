use serde::{Deserialize, Serialize};

use crate::cmd::Command;

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub enum Request {
    Init { nodes: usize, me: usize },
    Command(Command),
}

impl From<Request> for String {
    fn from(value: Request) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}

impl From<String> for Request {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap()
    }
}
