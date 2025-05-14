use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    event::{
        driver::EventDriver,
        info::{EventInfo, TcpEvent, TcpEventKind, TcpMessage},
        outcome::{EventOutcome, EventOutcomeKind},
    },
    log, send_local, sleep, spawn,
    tcp::packet::{TcpPacket, TcpPacketKind},
    Address, HashType, Node, Process, System, TcpListener, TcpStream,
};

use super::driver::TestEventDriver;

////////////////////////////////////////////////////////////////////////////////

struct Connector {}

impl Process for Connector {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, content: String) {
        let addr: Address = content.into();
        spawn(async move {
            log("connecting");
            let mut s = TcpStream::connect(&addr).await.unwrap();
            log("connected");
            let result = s.recv(&mut [0u8; 100]).await;
            assert!(result.is_err());
            log("disconnected");
            send_local("disconnected");
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Receiver {}

impl Process for Receiver {
    fn on_message(&mut self, _from: Address, _content: String) {
        unreachable!()
    }

    fn on_local_message(&mut self, _content: String) {
        spawn(async move {
            let s = TcpListener::listen().await.unwrap();
            log("connected");
            println!("sleeping...");
            sleep(Duration::from_secs(1)).await;
            println!("slept");
            drop(s);
            log("dropped");
        });
    }

    fn hash(&self) -> HashType {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn disconnect_delayed() {
    let driver = TestEventDriver::default();
    let driver = Rc::new(RefCell::new(driver));

    let system = System::new_default_net(&(driver.clone() as Rc<RefCell<dyn EventDriver>>));
    let system = system.handle();

    let mut sender = Node::new("sender");
    let sender_proc_addr = sender.add_proc("p", Connector {}).unwrap().address();
    system.add_node(sender).unwrap();

    let mut recv = Node::new("recv");
    let recv_proc_addr = recv.add_proc("p", Receiver {}).unwrap().address();
    system.add_node(recv).unwrap();

    system.send_local(&recv_proc_addr, "...").unwrap();
    system
        .send_local(&sender_proc_addr, recv_proc_addr.to_string())
        .unwrap();

    let o1 = {
        assert_eq!(driver.borrow().events.len(), 1);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpMessage(TcpMessage {
                packet: TcpPacket {
                    kind: TcpPacketKind::Connect(),
                    ..
                },
                ..
            },)
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpPacketDelivered(),
            time: event.time,
        }
    };

    // connect
    system.handle_event_outcome(o1);

    let o2 = {
        assert_eq!(driver.borrow().events.len(), 2);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpMessage(TcpMessage {
                packet: TcpPacket {
                    kind: TcpPacketKind::Ack(),
                    ..
                },
                ..
            })
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpPacketDelivered(),
            time: event.time,
        }
    };

    // connect ack
    system.handle_event_outcome(o2);

    let o3 = {
        assert_eq!(driver.borrow().events.len(), 1);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(event.info, EventInfo::Timer(..)));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TimerFired(),
            time: event.time,
        }
    };

    // timer fired
    system.handle_event_outcome(o3);

    let o4 = {
        assert_eq!(driver.borrow().events.len(), 1);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpEvent(TcpEvent {
                kind: TcpEventKind::SenderDropped,
                ..
            })
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpPacketDelivered(),
            time: event.time,
        }
    };

    // sender not disconnected yet
    assert!(system.read_locals("sender", "p").unwrap().is_empty());

    // disonnect
    system.handle_event_outcome(o4);

    println!("{}", system.log());
    assert_eq!(system.read_locals("sender", "p").unwrap().len(), 1);
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn disconnect_on_crash_delayed() {
    let driver = TestEventDriver::default();
    let driver = Rc::new(RefCell::new(driver));

    let system = System::new_default_net(&(driver.clone() as Rc<RefCell<dyn EventDriver>>));
    let system = system.handle();

    let mut sender = Node::new("sender");
    let sender_proc_addr = sender.add_proc("p", Connector {}).unwrap().address();
    system.add_node(sender).unwrap();

    let mut recv = Node::new("recv");
    let recv_proc_addr = recv.add_proc("p", Receiver {}).unwrap().address();
    system.add_node(recv).unwrap();

    system.send_local(&recv_proc_addr, "...").unwrap();
    system
        .send_local(&sender_proc_addr, recv_proc_addr.to_string())
        .unwrap();

    let o1 = {
        assert_eq!(driver.borrow().events.len(), 1);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpMessage(TcpMessage {
                packet: TcpPacket {
                    kind: TcpPacketKind::Connect(),
                    ..
                },
                ..
            },)
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpPacketDelivered(),
            time: event.time,
        }
    };

    // connect
    system.handle_event_outcome(o1);

    let o2 = {
        assert_eq!(driver.borrow().events.len(), 2);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpMessage(TcpMessage {
                packet: TcpPacket {
                    kind: TcpPacketKind::Ack(),
                    ..
                },
                ..
            },)
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpPacketDelivered(),
            time: event.time,
        }
    };

    // connect ack
    system.handle_event_outcome(o2);

    // crash recv
    system.crash_node("recv").unwrap();

    println!("node crashed");

    let o3 = {
        assert_eq!(driver.borrow().events.len(), 1);
        let event = driver.borrow_mut().take(0);
        assert!(matches!(
            event.info,
            EventInfo::TcpEvent(TcpEvent {
                kind: TcpEventKind::SenderDropped,
                ..
            })
        ));
        EventOutcome {
            event_id: event.id,
            kind: EventOutcomeKind::TcpEventHappen(TcpEventKind::SenderDropped.tcp_result()),
            time: event.time,
        }
    };

    // node already dropped, local message was not sent,
    // that confirms delay between drop and receiver notification.
    assert!(system.read_locals("sender", "p").unwrap().is_empty());

    // disonnect
    system.handle_event_outcome(o3);

    println!("{}", system.log());
    assert_eq!(system.read_locals("sender", "p").unwrap().len(), 1);
}
