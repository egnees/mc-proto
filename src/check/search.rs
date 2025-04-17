use crate::{search, sim::system::SystemHandle};

////////////////////////////////////////////////////////////////////////////////

pub struct ApplyFnWrapper<F>
where
    F: search::control::ApplyFn,
{
    inner: F,
}

impl<F> ApplyFnWrapper<F>
where
    F: search::control::ApplyFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> crate::search::control::ApplyFunctor for ApplyFnWrapper<F>
where
    F: search::control::ApplyFn,
{
    fn apply(&self, sys: SystemHandle) {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn crate::search::control::ApplyFunctor> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}
