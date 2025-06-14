use crate::{model, real};

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_real() -> bool {
    real::context::Context::installed()
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_sim() -> bool {
    model::context::Context::installed()
}
