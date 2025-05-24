use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////

pub struct RpcResponse {
    pub content: Vec<u8>,
}

impl RpcResponse {
    pub fn unpack<'a, T: Deserialize<'a>>(&'a self) -> Option<T> {
        serde_json::from_slice(self.content.as_slice()).ok()
    }
}
