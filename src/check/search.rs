use crate::{
    search,
    system::sys::{StateHandle, System},
};

////////////////////////////////////////////////////////////////////////////////

pub struct BuildFnWrapper<F>
where
    F: search::control::BuildFn,
{
    inner: F,
}

impl<F> BuildFnWrapper<F>
where
    F: search::control::BuildFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> crate::search::control::Builder for BuildFnWrapper<F>
where
    F: search::control::BuildFn,
{
    fn build(&self) -> System {
        (self.inner)()
    }

    fn clone(&self) -> Box<dyn crate::search::control::Builder> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct InvariantFnWrapper<F>
where
    F: search::control::InvariantFn,
{
    inner: F,
}

impl<F> InvariantFnWrapper<F>
where
    F: search::control::InvariantFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> crate::search::control::InvariantChecker for InvariantFnWrapper<F>
where
    F: search::control::InvariantFn,
{
    fn check(&self, sys: StateHandle) -> Result<(), String> {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn crate::search::control::InvariantChecker> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GoalFnWrapper<F>
where
    F: search::control::GoalFn,
{
    inner: F,
}

impl<F> GoalFnWrapper<F>
where
    F: search::control::GoalFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> crate::search::control::GoalChecker for GoalFnWrapper<F>
where
    F: search::control::GoalFn,
{
    fn check(&self, sys: StateHandle) -> bool {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn crate::search::control::GoalChecker> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PruneFnWrapper<F>
where
    F: search::control::PruneFn,
{
    inner: F,
}

impl<F> PruneFnWrapper<F>
where
    F: search::control::PruneFn,
{
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F> crate::search::control::Pruner for PruneFnWrapper<F>
where
    F: search::control::PruneFn,
{
    fn check(&self, sys: StateHandle) -> bool {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn crate::search::control::Pruner> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}

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
    fn apply(&self, sys: &mut System) {
        (self.inner)(sys)
    }

    fn clone(&self) -> Box<dyn crate::search::control::ApplyFunctor> {
        let clone = Self {
            inner: self.inner.clone(),
        };
        Box::new(clone)
    }
}
