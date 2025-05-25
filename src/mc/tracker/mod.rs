#[allow(unused)]
mod floyd;

#[allow(unused)]
mod graph;

mod moore;

////////////////////////////////////////////////////////////////////////////////

pub trait EventTracker<T>: Sized {
    fn add_event(&mut self, prev: usize, min_offset: T, max_offset: T) -> usize;

    fn event_happen(self, event: usize) -> Option<Self>;

    fn pending_events(&self) -> impl Iterator<Item = usize> + '_;

    fn next_events(&self) -> impl Iterator<Item = (usize, Self)> + '_;
}

////////////////////////////////////////////////////////////////////////////////

pub use moore::MooreEventTracker;

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
