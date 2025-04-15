use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    event1::{
        driver::EventDriver,
        outcome::{EventOutcome, EventOutcomeKind},
        time::TimeSegment,
        Event,
    },
    send_local, send_message, sleep, spawn, Address, HashType, NetConfig, Node, Process, System,
};

////////////////////////////////////////////////////////////////////////////////

struct Pinger {
    receiver: Address,
}

impl Process for Pinger {
    fn on_message(&mut self, from: Address, content: String) {
        assert_eq!(from, self.receiver);
        send_local(content);
    }

    fn on_local_message(&mut self, content: String) {
        send_message(&self.receiver, content);
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

struct Ponger {}

impl Process for Ponger {
    fn on_message(&mut self, from: Address, content: String) {
        send_message(&from, content.clone());
        send_local(content);
    }

    fn on_local_message(&mut self, _content: String) {
        unreachable!()
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct Sleeper {}

impl Sleeper {
    fn new() -> Self {
        Default::default()
    }
}

impl Process for Sleeper {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let ms = u64::from_str_radix(content.as_str(), 10).unwrap();
        spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            send_local(content);
        });
    }

    fn hash(&self) -> HashType {
        unreachable!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct EventCollector {
    events: Vec<Event>,
}

impl EventDriver for EventCollector {
    fn register_event(&mut self, event: &Event) {
        self.events.push(event.clone())
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
    let net_delays = TimeSegment::new(Duration::from_millis(100), Duration::from_millis(200));
    let net = NetConfig::new(net_delays.from, net_delays.to).unwrap();
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
    assert_eq!(collector.borrow().events[0].time, net_delays);
    let id = collector.borrow().events[0].id;

    // deliver udp message
    handle.handle_event_outcome(EventOutcome {
        event_id: id,
        kind: EventOutcomeKind::UdpMessageDelivered(),
    });

    // check system time
    assert_eq!(handle.time(), net_delays);

    // check ponger got message and send local msg
    let locals = handle.read_locals("n2", "ponger").unwrap();
    assert_eq!(locals.len(), 1);
    assert_eq!(locals[0], "1");
    assert_eq!(collector.borrow().events.len(), 2);
    let double_net_delays = net_delays.shift_range(net_delays.from, net_delays.to);
    assert_eq!(collector.borrow().events[1].time, double_net_delays);

    // deliver second msg
    let id = collector.borrow().events[1].id;
    handle.handle_event_outcome(EventOutcome {
        event_id: id,
        kind: EventOutcomeKind::UdpMessageDelivered(),
    });

    // check system time
    assert_eq!(handle.time(), double_net_delays);

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
    let net_delays = TimeSegment::new(Duration::from_millis(100), Duration::from_millis(200));
    let net = NetConfig::new(net_delays.from, net_delays.to).unwrap();
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
    let ms100 = Duration::from_millis(100);
    assert_eq!(
        collector.borrow().events[0].time,
        TimeSegment::new(ms100, ms100)
    );
    let first_sleep_id = collector.borrow().events[0].id;

    // sleep again
    handle.send_local(&sleeper.address(), "200").unwrap();
    assert_eq!(collector.borrow().events.len(), 2);
    let ms200 = Duration::from_millis(200);
    assert_eq!(
        collector.borrow().events[1].time,
        TimeSegment::new(ms200, ms200)
    );
    let second_sleep_id = collector.borrow().events[1].id;

    // first wakeup
    handle.handle_event_outcome(EventOutcome {
        event_id: first_sleep_id,
        kind: EventOutcomeKind::TimerFired(),
    });

    assert_eq!(handle.time(), TimeSegment::new(ms100, ms100));

    // second wakeup
    handle.handle_event_outcome(EventOutcome {
        event_id: second_sleep_id,
        kind: EventOutcomeKind::TimerFired(),
    });
    assert_eq!(handle.time(), TimeSegment::new(ms200, ms200));
    assert_eq!(sleeper.drain_locals(), ["100", "200"]);

    // one more local
    handle.send_local(&sleeper.address(), "300").unwrap();
    let ms500 = Duration::from_millis(500);
    assert_eq!(collector.borrow().events.len(), 3);
    let thrid_sleep_id = collector.borrow().events[2].id;
    handle.handle_event_outcome(EventOutcome {
        event_id: thrid_sleep_id,
        kind: EventOutcomeKind::TimerFired(),
    });
    assert_eq!(handle.time(), TimeSegment::new(ms500, ms500));
    assert_eq!(sleeper.drain_locals(), ["300"]);

    // print log
    println!("{}", handle.log());
}
