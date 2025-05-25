//! Provides utility for comfortable cancel async tasks.

use crate::JoinHandle;

////////////////////////////////////////////////////////////////////////////////

/// Represents wrapper over the async activity handles [`crate::JoinHandle`],
/// which cancels them on drops (it calls to [crate::JoinHandle::abort] on drop).
pub struct CancelSet<T> {
    handles: Vec<JoinHandle<T>>,
}

impl<T> FromIterator<JoinHandle<T>> for CancelSet<T> {
    fn from_iter<I: IntoIterator<Item = JoinHandle<T>>>(iter: I) -> Self {
        let handles = iter.into_iter().collect();
        Self { handles }
    }
}

impl<T> Drop for CancelSet<T> {
    fn drop(&mut self) {
        self.handles.iter_mut().for_each(|h| h.abort());
    }
}
