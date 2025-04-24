use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, PartialEq)]
pub struct VecClock(Vec<usize>);

impl VecClock {
    pub fn new(proc: usize) -> Self {
        Self((0..proc).map(|_| 0).collect())
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

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub vc: VecClock,
}

////////////////////////////////////////////////////////////////////////////////

pub trait Mailman {
    fn deliver(&mut self, msg: String);
}

////////////////////////////////////////////////////////////////////////////////

pub struct MessageRegister {
    mailman: Box<dyn Mailman>,
    vc: VecClock,
    not_delivered: Vec<Message>,
}

impl MessageRegister {
    pub fn new(proc: usize, mailman: Box<dyn Mailman>) -> Self {
        Self {
            mailman,
            vc: VecClock::new(proc),
            not_delivered: Default::default(),
        }
    }

    pub fn register(&mut self, msg: Message, from: usize) {
        self.vc.inc(from);
        self.not_delivered.push(msg);
        self.deliver_ready();
    }

    fn deliver_ready(&mut self) {
        let mut ready = Vec::new();
        for i in 0..self.not_delivered.len() {
            let cmp = self.not_delivered[i].vc.partial_cmp(&self.vc);
            match &cmp {
                Some(std::cmp::Ordering::Equal) | Some(std::cmp::Ordering::Less) => {
                    ready.push(i);
                }
                _ => {}
            }
        }
        for r in ready.iter().rev() {
            self.not_delivered.remove(*r);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
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
}
