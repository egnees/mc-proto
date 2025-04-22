use super::Event;

////////////////////////////////////////////////////////////////////////////////

pub trait EventDriver {
    fn register_event(&mut self, event: &Event);

    fn cancel_event(&mut self, event: &Event);
}
