use std::{cell::RefCell, rc::Rc, time::Duration};

use super::common::{Pinger, Ponger, Sleeper};

use crate::{
    model::event::{
        driver::EventDriver,
        outcome::{EventOutcome, EventOutcomeKind},
        Event,
    },
    model::net::Config as NetConfig,
    model::node::Node,
    model::system::System,
    Address,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct EventCollector {
    events: Vec<Event>,
}

impl EventDriver for EventCollector {
    fn register_event(&mut self, event: &Event, _min_offset: Duration, _max_offset: Duration) {
        self.events.push(Event {
            id: event.id,
            time: event.time,
            info: event.info.clone(),
            on_happen: None,
        })
    }

    fn cancel_event(&mut self, _event: &Event) {
        unreachable!()
    }

    fn hash_pending(&self) -> u64 {
        0
    }
}

impl EventCollector {
    fn new() -> Self {
        Default::default()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic_net() {
    // make driver
    let collector = {
        let d = EventCollector::new();
        Rc::new(RefCell::new(d))
    };

    // add net and system
    let min_net_delay = Duration::from_millis(100);
    let max_net_delay = Duration::from_millis(200);
    let net = NetConfig::new(min_net_delay, max_net_delay).unwrap();
    let system = System::new(&net, &(collector.clone() as Rc<RefCell<dyn EventDriver>>));

    // add nodes
    let mut n2 = Node::new("n2");
    let ponger = n2.add_proc("ponger", Ponger {}).unwrap();

    let mut n1 = Node::new("n1");
    n1.add_proc(
        "pinger",
        Pinger {
            receiver: ponger.address(),
        },
    )
    .unwrap();
    let handle = system.handle();
    handle.add_node(n1).unwrap();
    handle.add_node(n2).unwrap();

    // send local to first node
    handle
        .send_local(&Address::new("n1", "pinger"), "1")
        .unwrap();

    // check system emitted msg event
    assert_eq!(collector.borrow().events.len(), 1);
    assert_eq!(collector.borrow().events[0].time, Duration::ZERO);
    let id = collector.borrow().events[0].id;

    // deliver udp message
    handle.handle_event_outcome(EventOutcome {
        event_id: id,
        time: min_net_delay,
        kind: EventOutcomeKind::UdpMessageDelivered(),
    });

    // check system time
    assert_eq!(handle.time(), min_net_delay);

    // check ponger got message and send local msg
    let locals = handle.read_locals("n2", "ponger").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "1");
    assert_eq!(collector.borrow().events.len(), 2);
    let double_min_net_delay = min_net_delay * 2;
    assert_eq!(collector.borrow().events[1].time, min_net_delay);

    // deliver second msg
    let id = collector.borrow().events[1].id;
    handle.handle_event_outcome(EventOutcome {
        event_id: id,
        time: double_min_net_delay,
        kind: EventOutcomeKind::UdpMessageDelivered(),
    });

    // check system time
    assert_eq!(handle.time(), double_min_net_delay);

    // check first node deliver msg
    let locals = handle.read_locals("n1", "pinger").unwrap();
    assert_eq!(locals.len(), 1);

    // print log
    println!("{}", handle.log());
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn basic_sleep() {
    // make driver
    let collector = {
        let d = EventCollector::new();
        Rc::new(RefCell::new(d))
    };

    // add net and system
    let net = NetConfig::new(Duration::from_millis(100), Duration::from_millis(200)).unwrap();
    let system = System::new(&net, &(collector.clone() as Rc<RefCell<dyn EventDriver>>));

    // add nodes
    let mut n1 = Node::new("n1");
    let sleeper = n1.add_proc("sleeper", Sleeper::new()).unwrap();

    let handle = system.handle();
    handle.add_node(n1).unwrap();

    // send local to sleeper
    handle.send_local(&sleeper.address(), "100").unwrap();

    // check system emitted msg event
    assert_eq!(collector.borrow().events.len(), 1);
    assert_eq!(collector.borrow().events[0].time, Duration::ZERO);
    let first_sleep_id = collector.borrow().events[0].id;

    // sleep again
    handle.send_local(&sleeper.address(), "200").unwrap();
    assert_eq!(collector.borrow().events.len(), 2);

    assert_eq!(collector.borrow().events[1].time, Duration::ZERO);
    let second_sleep_id = collector.borrow().events[1].id;

    let ms100 = Duration::from_millis(100);
    let ms200 = Duration::from_millis(200);

    // first wakeup
    handle.handle_event_outcome(EventOutcome {
        event_id: first_sleep_id,
        time: ms100,
        kind: EventOutcomeKind::TimerFired(),
    });

    assert_eq!(handle.time(), ms100);

    // second wakeup
    handle.handle_event_outcome(EventOutcome {
        event_id: second_sleep_id,
        time: ms200,
        kind: EventOutcomeKind::TimerFired(),
    });
    assert_eq!(handle.time(), ms200);
    assert_eq!(sleeper.drain_locals(), ["100", "200"]);

    // one more local
    handle.send_local(&sleeper.address(), "300").unwrap();
    let ms500 = Duration::from_millis(500);
    assert_eq!(collector.borrow().events.len(), 3);
    let thrid_sleep_id = collector.borrow().events[2].id;
    handle.handle_event_outcome(EventOutcome {
        event_id: thrid_sleep_id,
        time: ms500,
        kind: EventOutcomeKind::TimerFired(),
    });
    assert_eq!(handle.time(), ms500);
    assert_eq!(sleeper.drain_locals(), ["300"]);

    // print log
    println!("{}", handle.log());
}
