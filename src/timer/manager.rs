use std::collections::HashMap;

use crate::util::oneshot;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct TimerManager {
    timers: HashMap<usize, oneshot::Sender<()>>,
}

////////////////////////////////////////////////////////////////////////////////

impl TimerManager {
    pub fn create(&mut self, id: usize) -> oneshot::Receiver<()> {
        let (rx, tx) = oneshot::channel();
        let prev = self.timers.insert(id, rx);
        assert!(prev.is_none());
        tx
    }

    pub fn remove(&mut self, id: usize) -> Option<oneshot::Sender<()>> {
        self.timers.remove(&id)
    }
}
