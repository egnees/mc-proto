use super::replicated::RepliactedU64;

////////////////////////////////////////////////////////////////////////////////

pub struct Term {
    value: RepliactedU64,
}

impl Term {
    pub async fn new() -> Self {
        Self {
            value: RepliactedU64::new("term.txt").await,
        }
    }

    pub fn get(&self) -> u64 {
        self.value.read()
    }

    pub fn set(&self, value: u64) -> mc::JoinHandle<()> {
        self.value.update(value)
    }

    pub fn increment(&self) -> (u64, mc::JoinHandle<()>) {
        self.value.increment()
    }
}
