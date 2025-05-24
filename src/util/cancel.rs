use crate::JoinHandle;

////////////////////////////////////////////////////////////////////////////////

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
