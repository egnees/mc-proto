use serde::Serialize;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RpcResponse {
    pub content: Vec<u8>,
}

impl RpcResponse {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn new_with_type<T: Serialize>(value: &T) -> serde_json::Result<Self> {
        let content = serde_json::to_vec(value)?;
        Ok(Self { content })
    }
}
