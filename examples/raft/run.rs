use std::error::Error;

use raft::{
    cmd::Response,
    proc::Raft,
    real::{self, http, registry},
    req::Request,
};
use tokio::sync::mpsc::unbounded_channel;

////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let Some(cfg) = std::env::args().nth(1) else {
        return Err("config path not specified".into());
    };
    let Some(cfg) = real::cfg::Config::from_file(cfg) else {
        return Err("invalid config".into());
    };

    let me = cfg.me;
    let nodes = cfg.routes.len();

    let mut route = dsbuild::real::RouteConfigBuilder::new();
    for (addr, sock) in cfg.routes.into_iter() {
        route = route.add(addr, sock);
    }
    let route = route.build();

    // Create node and init raft process
    let mut node = dsbuild::real::RealNode::new(me.node, cfg.my_id as u64, route, cfg.dir);
    let (sender, mut receiver) = node.add_proc(me.process, Raft::default()).unwrap();
    sender.send(Request::Init {
        nodes,
        me: cfg.my_id,
    });

    let (command_sender, mut command_receiver) = unbounded_channel();
    let reg = registry::CommandRegistry::new(cfg.my_id, command_sender).into_handle();

    // Spawn activity which passes user requests to the process
    node.spawn(async move {
        while let Some(cmd) = command_receiver.recv().await {
            let req = Request::Command(cmd);
            sender.send(req);
        }
    });

    // Spawn activity which listens to user requests
    node.spawn(http::serve(cfg.listen, reg.clone()));

    // Block on activity which waits for the process responses
    node.block_on(async move {
        while let Some(resp) = receiver.recv::<String>().await {
            let resp: Response = serde_json::from_str(&resp).unwrap();
            reg.on_response(resp).await;
        }
    });

    Ok(())
}
