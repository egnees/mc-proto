pub(crate) fn is_real() -> bool {
    crate::real::context::Context::installed()
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_sim() -> bool {
    crate::sim::context::Context::installed()
}
