use crate::{sim::system::SystemHandle, StateView};

////////////////////////////////////////////////////////////////////////////////

pub trait InvariantFn: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

impl<F> InvariantFn for F where
    F: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static
{
}

////////////////////////////////////////////////////////////////////////////////

pub trait PruneFn: Fn(StateView) -> bool + Send + Sync + Clone + 'static {}

impl<F> PruneFn for F where F: Fn(StateView) -> bool + Send + Sync + Clone + 'static {}

////////////////////////////////////////////////////////////////////////////////

pub trait GoalFn: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

impl<F> GoalFn for F where F: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

////////////////////////////////////////////////////////////////////////////////

pub trait ApplyFn: Fn(SystemHandle) + Send + Sync + Clone + 'static {}

impl<F> ApplyFn for F where F: Fn(SystemHandle) + Send + Sync + Clone + 'static {}

pub trait ApplyFunctor: Send + Sync {
    fn apply(&self, sys: SystemHandle);
    fn clone(&self) -> Box<dyn ApplyFunctor>;
}

impl Clone for Box<dyn ApplyFunctor> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}
