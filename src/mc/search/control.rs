use crate::{mc::StateView, model::system::SystemHandle};

////////////////////////////////////////////////////////////////////////////////

/// Check the system model invariants.
pub trait InvariantFn: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

impl<F> InvariantFn for F where
    F: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static
{
}

////////////////////////////////////////////////////////////////////////////////

/// Allows to prune not relevant states.
pub trait PruneFn: Fn(StateView) -> bool + Send + Sync + Clone + 'static {}

impl<F> PruneFn for F where F: Fn(StateView) -> bool + Send + Sync + Clone + 'static {}

////////////////////////////////////////////////////////////////////////////////

/// Represents predicate, which checks if the system model state achieves search goal
/// (see [crate::mc::ModelChecker::check] and [crate::mc::ModelChecker::collect])
pub trait GoalFn: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

impl<F> GoalFn for F where F: Fn(StateView) -> Result<(), String> + Send + Sync + Clone + 'static {}

////////////////////////////////////////////////////////////////////////////////

/// Allows to make some actions with the system model.
/// For example, it can be used to send local messages to process,
/// or crash some node.
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
