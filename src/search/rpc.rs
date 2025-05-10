use std::collections::{btree_map::Entry, BTreeMap};

use crate::Address;

////////////////////////////////////////////////////////////////////////////////

pub struct RpcMessageInfo {
    pub id: u64,
    pub from: Address,
    pub to: Address,
}

impl RpcMessageInfo {
    pub fn new(id: u64, from: Address, to: Address) -> Self {
        Self { id, from, to }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ReadyRpcRequestsFilter<M> {
    // (from, to) -> request_id
    req: BTreeMap<(Address, Address), (u64, M)>,
}

impl<M> ReadyRpcRequestsFilter<M> {
    pub fn new() -> Self {
        Self {
            req: Default::default(),
        }
    }

    pub fn add(&mut self, req: &RpcMessageInfo, meta: M) {
        let entry = self.req.entry((req.from.clone(), req.to.clone()));
        match entry {
            Entry::Occupied(mut e) => {
                if req.id < e.get().0 {
                    e.insert((req.id, meta));
                }
            }
            Entry::Vacant(e) => {
                e.insert((req.id, meta));
            }
        }
    }

    pub fn ready_packets(&self) -> impl Iterator<Item = &(u64, M)> + '_ {
        self.req.values()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::Address;

    use super::{ReadyRpcRequestsFilter, RpcMessageInfo};

    ////////////////////////////////////////////////////////////////////////////////

    pub fn a1() -> Address {
        Address::new("n1", "p1")
    }

    pub fn a2() -> Address {
        Address::new("n2", "p2")
    }

    pub fn a3() -> Address {
        Address::new("n3", "p3")
    }

    pub fn a4() -> Address {
        Address::new("n4", "p4")
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut filter = ReadyRpcRequestsFilter::new();
        filter.add(&RpcMessageInfo::new(0, a1(), a2()), ());
        filter.add(&RpcMessageInfo::new(1, a3(), a4()), ());
        let mut packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        packets.sort();
        assert_eq!(packets, [0, 1]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn select_request_with_min_id() {
        let mut filter = ReadyRpcRequestsFilter::new();
        filter.add(&RpcMessageInfo::new(2, a1(), a2()), ());
        filter.add(&RpcMessageInfo::new(1, a1(), a2()), ());
        filter.add(&RpcMessageInfo::new(3, a1(), a2()), ());
        let packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(packets, [1]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn different_dirrections() {
        let mut filter = ReadyRpcRequestsFilter::new();
        filter.add(&RpcMessageInfo::new(2, a1(), a2()), ());
        filter.add(&RpcMessageInfo::new(1, a2(), a1()), ());
        let mut packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        packets.sort();
        assert_eq!(packets, [1, 2]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn just_many() {
        let mut filter = ReadyRpcRequestsFilter::new();
        let mut add = |id, from, to| {
            filter.add(&RpcMessageInfo::new(id, from, to), ());
        };
        add(1, a1(), a2());
        add(2, a1(), a2());
        add(4, a2(), a1());
        add(6, a2(), a3());
        add(7, a2(), a4());
        add(5, a2(), a1());
        add(3, a2(), a1());
        add(0, a1(), a2());
        add(10, a2(), a4());
        add(8, a2(), a4());
        add(9, a2(), a3());
        let mut packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        packets.sort();
        assert_eq!(packets, [0, 3, 6, 7]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn consistent() {
        let get = || {
            let mut filter = ReadyRpcRequestsFilter::new();
            let mut add = |id, from, to| {
                filter.add(&RpcMessageInfo::new(id, from, to), ());
            };
            add(1, a1(), a2());
            add(2, a1(), a2());
            add(4, a2(), a1());
            add(6, a2(), a3());
            add(7, a2(), a4());
            add(5, a2(), a1());
            add(3, a2(), a1());
            add(0, a1(), a2());
            add(10, a2(), a4());
            add(8, a2(), a4());
            add(9, a2(), a3());
            filter
                .ready_packets()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        };

        let r = get();
        {
            let mut r = r.clone();
            r.sort();
            assert_eq!(r, [0, 3, 6, 7]);
        }

        for _ in 0..100 {
            let cur = get();
            assert_eq!(cur, r);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn meta() {
        let mut filter = ReadyRpcRequestsFilter::new();
        filter.add(&RpcMessageInfo::new(1, a1(), a2()), 123);
        filter.add(&RpcMessageInfo::new(0, a1(), a2()), 321);
        let packets = filter.ready_packets().collect::<Vec<_>>();
        assert_eq!(packets, [&(0, 321)]);
    }
}
