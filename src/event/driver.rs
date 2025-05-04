use super::{time::Time, Event};

////////////////////////////////////////////////////////////////////////////////

pub trait EventDriver {
    fn start_time(&self) -> Time;

    fn register_event(&mut self, event: &Event);

    fn cancel_event(&mut self, event: &Event);
}
