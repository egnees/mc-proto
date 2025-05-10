use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RpcResponse {
    pub id: u64,
    pub content: Vec<u8>,
}

impl RpcResponse {
    pub fn new(id: u64, content: Vec<u8>) -> Self {
        Self { id, content }
    }

    pub fn new_with_type<T: Serialize>(id: u64, value: &T) -> serde_json::Result<Self> {
        let content = serde_json::to_vec(value)?;
        Ok(Self::new(id, content))
    }

    pub fn unpack<'a, T: Deserialize<'a>>(&'a self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.content)
    }
}
