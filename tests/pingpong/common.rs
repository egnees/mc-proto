use std::{cell::RefCell, rc::Rc, time::Duration};

////////////////////////////////////////////////////////////////////////////////

pub fn make_build(
    min_packet_delay: Duration,
    max_packet_delay: Duration,
    ping: impl Fn() -> Rc<RefCell<dyn mc::Process>> + Clone + Sync + Send + 'static,
    pong: impl Fn() -> Rc<RefCell<dyn mc::Process>> + Clone + Sync + Send + 'static,
    locals: usize,
) -> impl mc::ApplyFn {
    move |sys| {
        // configure network
        sys.network()
            .set_delays(min_packet_delay, max_packet_delay)
            .unwrap();

        // configure first node
        let mut n1 = mc::Node::new("n1");
        n1.add_proc_by_ref("ping", ping()).unwrap();
        sys.add_node(n1).unwrap();

        // configure second node
        let mut n2 = mc::Node::new("n2");
        n2.add_proc_by_ref("pong", pong()).unwrap();
        sys.add_node(n2).unwrap();

        // send local messages to initiate requests
        for i in 0..locals {
            sys.send_local(&mc::Address::new("n1", "ping"), i.to_string())
                .unwrap();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_goal(locals: usize) -> impl mc::GoalFn {
    move |s: mc::StateView| {
        let s = s.system();
        let Some(mut ping_locals) = s.read_locals("n1", "ping") else {
            return Err("No such address n1:ping".into());
        };
        ping_locals.sort();

        let Some(mut pong_locals) = s.read_locals("n2", "pong") else {
            return Err("No such address n2:pong".into());
        };
        pong_locals.sort();

        let ref_locals = (0..locals).map(|n| n.to_string()).collect::<Vec<_>>();

        if ping_locals == ref_locals && pong_locals == ref_locals {
            Ok(())
        } else {
            Err("Goal not achieved".into())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_invariant(locals: usize) -> impl mc::InvariantFn {
    move |s: mc::StateView| {
        let s = s.system();
        if s.read_locals("n1", "ping").unwrap_or_default().len() <= locals
            && s.read_locals("n2", "pong").unwrap_or_default().len() <= locals
        {
            Ok(())
        } else {
            Err("too many locals".into())
        }
    }
}
