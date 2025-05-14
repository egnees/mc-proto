use super::replicated::RepliactedU64;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Term {
    value: RepliactedU64,
}

impl Term {
    pub async fn get(&mut self) -> mc::FsResult<u64> {
        self.value.read().await
    }

    pub async fn set(&mut self, value: u64) -> mc::FsResult<()> {
        self.value.update(value).await
    }
}
