pub trait BuildFn: Fn(mc::SystemHandle, usize) + Send + Sync + Clone + 'static {}

impl<F: Fn(mc::SystemHandle, usize) + Send + Sync + Clone + 'static> BuildFn for F {}

////////////////////////////////////////////////////////////////////////////////

pub fn send_local(s: mc::SystemHandle, node: usize, msg: impl Into<String>) -> bool {
    let proc = node;
    let address = format!("{}:{}", node, proc).into();
    s.send_local(&address, msg).is_ok()
}
