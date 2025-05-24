use std::error::Error;

use mc::RouteConfigBuilder;
use raft::{proc::Raft, real, req::Request};

////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let Some(cfg) = std::env::args().skip(1).next() else {
        return Err("config path not specified".into());
    };
    let Some(cfg) = real::cfg::Config::from_file(cfg) else {
        return Err("invalid config".into());
    };

    let me = cfg.me;
    let nodes = cfg.routes.len();

    let mut route = RouteConfigBuilder::new();
    for (addr, sock) in cfg.routes.into_iter() {
        route = route.add(addr, sock);
    }
    let route = route.build();

    let mut node = mc::RealNode::new(me.node, 123, route, cfg.dir);
    let (sender, mut receiver) = node.add_proc(me.process, Raft::default()).unwrap();
    sender.send(Request::Init {
        nodes,
        me: cfg.my_id,
    });
    node.block_on(async move {
        let _ = receiver.recv::<String>().await;
    });
    Ok(())
}
