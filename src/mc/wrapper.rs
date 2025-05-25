use crate::model::system::SystemHandle;

use super::search::control;

////////////////////////////////////////////////////////////////////////////////

pub struct ApplyFnWrapper<F>
where
    F: control::ApplyFn,
{
    inner: F,
}

impl<F> ApplyFnWrapper<F>
where
    F: control::ApplyFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> control::ApplyFunctor for ApplyFnWrapper<F>
where
    F: control::ApplyFn,
{
    fn apply(&self, sys: SystemHandle) {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn control::ApplyFunctor> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}
