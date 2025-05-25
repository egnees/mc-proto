use std::time::Duration;

use dsbuild::detsim;
use raft::{
    addr::{PROCESS_NAME, RAFT_ROLE, make_addr, node},
    cmd::{Command, Response},
    proc::Raft,
    req::Request,
};

use crate::util::{leader, log_equals};

////////////////////////////////////////////////////////////////////////////////

pub struct RaftWrapper {
    sim: detsim::Simulation,
    nodes: usize,
}

impl RaftWrapper {
    pub fn new(seed: u64, nodes: usize) -> Self {
        let wrapper = Self {
            sim: detsim::Simulation::new(seed),
            nodes,
        };
        for n in 0..nodes {
            wrapper.add_node(n);
        }
        wrapper
    }
}

impl RaftWrapper {
    fn add_node(&self, n: usize) {
        let node_name = node(n);
        let mut node = dsbuild::model::Node::new(&node_name);
        let proc = node.add_proc(PROCESS_NAME, Raft::default()).unwrap();
        let s = self.sim.system();
        s.add_node_with_role(node, RAFT_ROLE).unwrap();
        s.setup_fs(
            &node_name,
            Duration::from_millis(1),
            Duration::from_millis(3),
            4096,
        )
        .unwrap();
        s.send_local(
            &proc.address(),
            Request::Init {
                nodes: self.nodes,
                me: n,
            },
        )
        .unwrap();
    }

    pub fn system(&self) -> dsbuild::model::SystemHandle {
        self.sim.system()
    }

    pub fn step_until_leader_found(&self) -> usize {
        loop {
            let leader = self.find_leader();
            if let Some(leader) = leader {
                return leader;
            }
            self.sim.step(&detsim::StepConfig::no_drops());
        }
    }

    pub fn find_leader(&self) -> Option<usize> {
        leader(self.sim.system(), self.nodes)
            .map(|l| l.unwrap() as usize)
            .ok()
    }

    pub fn send_command(&self, to: usize, cmd: Command) -> Response {
        let id = cmd.id;
        let addr = make_addr(to);
        self.sim
            .system()
            .send_local(&addr, raft::req::Request::Command(cmd))
            .unwrap();
        let cfg = detsim::StepConfig::no_drops();
        self.sim.step_unti(
            |s| {
                let locals = s.read_locals(&addr.node, &addr.process).unwrap();
                locals
                    .iter()
                    .find(|s| {
                        let resp: Response = (*s).clone().into();
                        resp.id == id
                    })
                    .is_some()
            },
            &cfg,
        );
        let locals = self
            .sim
            .system()
            .read_locals(&addr.node, &addr.process)
            .unwrap();
        let s = locals
            .iter()
            .find(|s| {
                let resp: Response = (*s).clone().into();
                resp.id == id
            })
            .cloned()
            .unwrap();
        s.into()
    }

    pub fn step_until_log_equals(&self) -> Vec<Command> {
        self.sim.step_unti(
            |s| log_equals(s, self.nodes).is_ok(),
            &detsim::StepConfig::no_drops(),
        );
        log_equals(self.sim.system(), self.nodes).unwrap().unwrap()
    }

    pub fn shutdown_node(&self, n: usize) {
        self.sim.system().shutdown_node(node(n)).unwrap();
    }

    pub fn restart_node(&self, n: usize) {
        self.sim.system().restart_node(node(n)).unwrap();
        let proc = self
            .sim
            .system()
            .add_proc_on_node(node(n), PROCESS_NAME, Raft::default())
            .unwrap();
        self.sim
            .system()
            .send_local(
                &proc.address(),
                Request::Init {
                    nodes: self.nodes,
                    me: n,
                },
            )
            .unwrap();
    }

    pub fn make_many_steps(&self) {
        for _ in 0..100 {
            self.sim.step(&detsim::StepConfig::no_drops());
        }
    }
}
