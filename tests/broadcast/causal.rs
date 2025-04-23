use std::{collections::HashSet, hash::Hash};

use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct VecClock(Vec<usize>);

impl VecClock {
    pub fn new(proc_cnt: usize) -> Self {
        Self((0..proc_cnt).map(|_| 0).collect())
    }

    pub fn inc(&mut self, comp: usize) {
        self.0[comp] += 1;
    }
}

impl PartialOrd for VecClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0.len() != other.0.len() {
            None
        } else {
            let mut gt = false;
            let mut ls = false;
            for i in 0..self.0.len() {
                if self.0[i] < other.0[i] {
                    ls = true;
                }
                if self.0[i] > other.0[i] {
                    gt = true;
                }
            }
            if !ls && !gt {
                Some(std::cmp::Ordering::Equal)
            } else if ls && !gt {
                Some(std::cmp::Ordering::Less)
            } else if gt && !ls {
                Some(std::cmp::Ordering::Greater)
            } else {
                assert!(ls && gt);
                None
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub content: String,
    pub vc: VecClock,
    pub from: usize,
}

////////////////////////////////////////////////////////////////////////////////

pub trait Mailman {
    fn deliver(&mut self, msg: &str);
}

////////////////////////////////////////////////////////////////////////////////
pub struct MessageRegister {
    mailman: Box<dyn Mailman>,
    vc: VecClock,
    not_delivered: Vec<Message>,
    reg: Vec<Message>,
}

impl Hash for MessageRegister {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.reg.iter().for_each(|m| m.content.hash(state));
    }
}

impl MessageRegister {
    pub fn new(proc_cnt: usize, mailman: impl Mailman + 'static) -> Self {
        Self {
            mailman: Box::new(mailman),
            vc: VecClock::new(proc_cnt),
            not_delivered: Default::default(),
            reg: Default::default(),
        }
    }

    pub fn vc(&self) -> &VecClock {
        &self.vc
    }

    pub fn register(&mut self, msg: Message) {
        assert_eq!(msg.vc.0.len(), self.vc.0.len());
        self.vc.inc(msg.from);
        self.reg.push(msg.clone());
        self.not_delivered.push(msg);
        self.deliver_ready();
    }

    fn deliver_ready(&mut self) {
        let mut ready = Vec::new();
        let mut to_deliver = Vec::new();
        for i in 0..self.not_delivered.len() {
            let cmp = self.not_delivered[i].vc.partial_cmp(&self.vc);
            match &cmp {
                Some(std::cmp::Ordering::Equal) | Some(std::cmp::Ordering::Less) => {
                    ready.push(i);
                    to_deliver.push(self.not_delivered[i].clone());
                }
                _ => {}
            }
        }

        ready.iter().rev().for_each(|r| {
            self.not_delivered.remove(*r);
        });

        to_deliver.sort_by(|a, b| a.vc.0.iter().sum::<usize>().cmp(&b.vc.0.iter().sum()));
        to_deliver
            .iter()
            .for_each(|m| self.mailman.deliver(m.content.as_str()));
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn make_message(content: String, me: usize, mut vc: VecClock) -> Message {
    vc.inc(me);
    Message {
        content,
        vc,
        from: me,
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CausalChecker {
    orders: HashSet<(String, String)>,
    deliver: Vec<HashSet<String>>,
    send: Vec<Vec<String>>,
}

impl CausalChecker {
    pub fn new(nodes: usize) -> Self {
        Self {
            orders: Default::default(),
            deliver: vec![HashSet::new(); nodes],
            send: vec![Vec::new(); nodes],
        }
    }

    pub fn deliver(&mut self, node: usize, msg: impl Into<String>) -> Result<(), String> {
        // check orderings
        let msg: String = msg.into();
        for before in self
            .orders
            .iter()
            .filter(|(_before, after)| *after == msg)
            .map(|(before, _after)| before)
        {
            if !self.deliver[node].contains(before) {
                return Err(format!("Causal order violation: deliver message {msg:?} on node {node}, but message {before:?} not delivered yet"));
            }
        }
        self.deliver[node].insert(msg);
        Ok(())
    }

    pub fn send(&mut self, node: usize, msg: impl Into<String>) {
        // msg is sequenced after all send and recv by node
        let msg: String = msg.into();
        self.send[node].iter().for_each(|s| {
            self.orders.insert((s.to_string(), msg.clone()));
        });
        self.send[node].push(msg.clone());
        self.deliver[node].iter().for_each(|s| {
            self.orders.insert((s.to_string(), msg.clone()));
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn vector_clock_cmp() {
        let v1 = VecClock(vec![1, 2, 0, 3]);
        let v2 = VecClock(vec![2, 1, 1, 4]);
        let v3 = VecClock(vec![2, 3, 1, 5]);
        let v4 = VecClock(vec![2, 3, 1, 5]);

        assert!(v1.partial_cmp(&v2).is_none());
        assert!(v2.partial_cmp(&v1).is_none());

        assert_eq!(v3.partial_cmp(&v1).unwrap(), std::cmp::Ordering::Greater);
        assert_eq!(v1.partial_cmp(&v3).unwrap(), std::cmp::Ordering::Less);

        assert_eq!(v3.partial_cmp(&v4).unwrap(), std::cmp::Ordering::Equal);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn register() {
        let store = Rc::new(RefCell::new(Vec::new()));

        struct Mail {
            store: Rc<RefCell<Vec<String>>>,
        }
        impl Mailman for Mail {
            fn deliver(&mut self, msg: &str) {
                self.store.borrow_mut().push(msg.to_string());
            }
        }

        let mut register = MessageRegister::new(
            3,
            Mail {
                store: store.clone(),
            },
        );

        let m1 = Message {
            content: "hello".into(),
            vc: VecClock(vec![0, 1, 1]),
            from: 1,
        };
        register.register(m1);
        assert!(store.borrow().is_empty());

        let prev_m = Message {
            content: "hello0".into(),
            vc: VecClock(vec![0, 0, 1]),
            from: 2,
        };
        register.register(prev_m);
        assert_eq!(store.borrow().len(), 2);

        let last_m = make_message("hello1".into(), 0, register.vc().clone());
        register.register(last_m);
        assert_eq!(store.borrow().len(), 3);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn order() {
        let store = Rc::new(RefCell::new(Vec::new()));

        struct Mail {
            store: Rc<RefCell<Vec<String>>>,
        }
        impl Mailman for Mail {
            fn deliver(&mut self, msg: &str) {
                self.store.borrow_mut().push(msg.to_string());
            }
        }

        let mut register = MessageRegister::new(
            3,
            Mail {
                store: store.clone(),
            },
        );

        let m1 = Message {
            content: "hello3".into(),
            vc: VecClock(vec![1, 1, 1]),
            from: 0,
        };
        register.register(m1);
        assert!(store.borrow().is_empty());

        let m2 = Message {
            content: "hello1".into(),
            vc: VecClock(vec![0, 1, 0]),
            from: 2,
        };
        register.register(m2);
        assert!(store.borrow().is_empty());

        let m3 = Message {
            content: "hello2".into(),
            vc: VecClock(vec![0, 1, 1]),
            from: 1,
        };
        register.register(m3);
        assert_eq!(store.borrow().len(), 3);

        // check order
        assert_eq!(store.borrow().first().unwrap(), "hello1");
        assert_eq!(store.borrow()[1], "hello2");
        assert_eq!(store.borrow().last().unwrap(), "hello3");
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn nodes_sim() {
        struct Mail {
            delivered: Rc<RefCell<usize>>,
        }

        impl Mailman for Mail {
            fn deliver(&mut self, _msg: &str) {
                *self.delivered.borrow_mut() += 1;
            }
        }

        struct Node {
            reg: MessageRegister,
            delivered: Rc<RefCell<usize>>,
            me: usize,
        }

        impl Node {
            fn recv(&mut self, msg: Message) {
                self.reg.register(msg);
            }

            fn bcast(&mut self) -> Message {
                let content = "123".into();
                let msg = make_message(content, self.me, self.reg.vc().clone());
                self.recv(msg.clone());
                msg
            }

            fn delivered(&self) -> usize {
                *self.delivered.borrow()
            }

            fn new(node_cnt: usize, me: usize) -> Self {
                let d = Rc::new(RefCell::new(0));
                let mail = Mail {
                    delivered: d.clone(),
                };
                let reg = MessageRegister::new(node_cnt, mail);
                Self {
                    reg,
                    delivered: d,
                    me,
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////////////

        let nodes = 4;

        let mut n0 = Node::new(nodes, 0);
        let mut n1 = Node::new(nodes, 1);
        let mut n2 = Node::new(nodes, 2);
        let mut n3 = Node::new(nodes, 3);

        let m1 = n0.bcast();
        assert_eq!(n0.delivered(), 1);

        let m2 = n1.bcast();
        assert_eq!(n1.delivered(), 1);

        // m1 and m2 are concurrent
        n2.recv(m1.clone());
        n2.recv(m2.clone());
        assert_eq!(n2.delivered(), 2);

        // m3 is sequenced after m1 and m2
        let m3 = n2.bcast();
        assert_eq!(n2.delivered(), 3); // m1, m2, m3

        n3.recv(m3.clone());
        assert_eq!(n3.delivered(), 0);

        // m2 delivery
        n3.recv(m2.clone());
        assert_eq!(n3.delivered(), 1);

        // deliver all
        n3.recv(m1);
        assert_eq!(n3.delivered(), 3);

        // m3 can not be delivered without m2
        n0.recv(m3);
        assert_eq!(n0.delivered(), 1);

        n0.recv(m2);
        assert_eq!(n0.delivered(), 3);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn serde() {
        let msg = Message {
            content: "123".into(),
            vc: VecClock(vec![3, 2, 1]),
            from: 0,
        };
        let ser = serde_json::to_string(&msg).unwrap();
        println!("{}", ser);
        assert!(!ser.contains(|c: char| c == '\n' || c.is_whitespace()));
        let deser: Message = serde_json::from_str(ser.as_str()).unwrap();
        assert_eq!(deser, msg);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn causal_checker_1() {
        let mut checker = CausalChecker::new(4);
        checker.send(0, "m1");
        checker.deliver(0, "m1").unwrap();
        checker.send(1, "m2");
        checker.deliver(1, "m2").unwrap();
        checker.deliver(2, "m1").unwrap();
        checker.deliver(2, "m2").unwrap();
        checker.send(2, "m3");
        checker.deliver(2, "m3").unwrap();

        // now m1, m2 are before m3

        assert!(checker.deliver(3, "m3").is_err());
        assert!(checker.deliver(0, "m3").is_err());
        assert!(checker.deliver(1, "m3").is_err());

        checker.deliver(3, "m1").unwrap();
        checker.deliver(3, "m2").unwrap();
        checker.deliver(3, "m3").unwrap();
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn causal_checker_2() {
        let mut checker = CausalChecker::new(2);
        checker.send(0, "m1");
        checker.send(1, "m2");

        checker.deliver(0, "m2").unwrap();
        checker.deliver(1, "m1").unwrap();

        checker.deliver(0, "m1").unwrap();
        checker.deliver(1, "m2").unwrap();
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[should_panic]
    #[test]
    fn causal_checker_3() {
        let mut checker = CausalChecker::new(2);
        checker.send(0, "m1");
        checker.send(0, "m2");

        checker.deliver(1, "m2").unwrap();
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[should_panic]
    #[test]
    fn causal_checker_4() {
        let mut checker = CausalChecker::new(1);
        checker.send(0, "m1");
        checker.send(0, "m2");

        checker.deliver(0, "m2").unwrap();
    }
}
