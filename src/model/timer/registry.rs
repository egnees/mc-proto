use std::time::Duration;

use crate::{util::oneshot, Address};

////////////////////////////////////////////////////////////////////////////////

pub trait TimerRegistry {
    fn register_timer(
        &mut self,
        min_duration: Duration,
        max_duration: Duration,
        with_sleep: bool,
        proc: Address,
    ) -> (usize, oneshot::Receiver<()>);

    fn cancel_timer(&mut self, id: usize, proc: Address);
}
