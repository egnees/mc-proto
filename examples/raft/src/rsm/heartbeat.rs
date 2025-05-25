use std::time::Duration;

use super::state::StateHandle;
use crate::addr;

////////////////////////////////////////////////////////////////////////////////

pub async fn send_heartbeats(state: StateHandle) {
    dsbuild::log("send_heartbeats");
    let nodes = state.nodes();
    let me = state.me();
    loop {
        let hb = state.make_heartbeat();
        let it = addr::iter_others(nodes, me).map(|n| {
            let hb = hb.clone();
            dsbuild::spawn(async move {
                let _ = hb.send(n).await;
            })
        });
        let _s = dsbuild::util::cancel::CancelSet::from_iter(it);
        let _ = dsbuild::set_timer(Duration::from_millis(150)).await;
    }
}
