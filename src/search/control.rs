use crate::system::sys::{StateHandle, System};

////////////////////////////////////////////////////////////////////////////////

pub trait InvariantFn:
    Fn(StateHandle) -> Result<(), String> + Send + Sync + Clone + 'static
{
}

impl<F> InvariantFn for F where
    F: Fn(StateHandle) -> Result<(), String> + Send + Sync + Clone + 'static
{
}

pub trait InvariantChecker: Send + Sync {
    fn check(&self, sys: StateHandle) -> Result<(), String>;
    fn clone(&self) -> Box<dyn InvariantChecker>;
}

impl Clone for Box<dyn InvariantChecker> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait PruneFn: Fn(StateHandle) -> bool + Send + Sync + Clone + 'static {}

impl<F> PruneFn for F where F: Fn(StateHandle) -> bool + Send + Sync + Clone + 'static {}

pub trait Pruner: Send + Sync {
    fn check(&self, sys: StateHandle) -> bool;
    fn clone(&self) -> Box<dyn Pruner>;
}

impl Clone for Box<dyn Pruner> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait GoalFn: Fn(StateHandle) -> bool + Send + Sync + Clone + 'static {}

impl<F> GoalFn for F where F: Fn(StateHandle) -> bool + Send + Sync + Clone + 'static {}

pub trait GoalChecker: Send + Sync {
    fn check(&self, sys: StateHandle) -> bool;
    fn clone(&self) -> Box<dyn GoalChecker>;
}

impl Clone for Box<dyn GoalChecker> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait BuildFn: Fn() -> System + Send + Sync + Clone + 'static {}

impl<F> BuildFn for F where F: Fn() -> System + Send + Sync + Clone + 'static {}

pub trait Builder: Send + Sync {
    fn build(&self) -> System;
    fn clone(&self) -> Box<dyn Builder>;
}

impl Clone for Box<dyn Builder> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ApplyFn: Fn(&mut System) + Send + Sync + Clone + 'static {}

impl<F> ApplyFn for F where F: Fn(&mut System) + Send + Sync + Clone + 'static {}

pub trait ApplyFunctor: Send + Sync {
    fn apply(&self, sys: &mut System);
    fn clone(&self) -> Box<dyn ApplyFunctor>;
}

impl Clone for Box<dyn ApplyFunctor> {
    fn clone(&self) -> Self {
        self.as_ref().clone()
    }
}
