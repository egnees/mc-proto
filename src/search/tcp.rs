use std::collections::{btree_map::Entry, BTreeMap};

////////////////////////////////////////////////////////////////////////////////

pub struct TcpPacketKind {
    pub tcp_packet_id: usize,
    pub stream: usize,
    pub dir: bool,
}

impl TcpPacketKind {
    #[allow(unused)]
    pub fn new(tcp_packet_id: usize, stream: usize, dir: bool) -> Self {
        Self {
            tcp_packet_id,
            stream,
            dir,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ReadyTcpPacketFilter<M> {
    // (stream, dir) -> tcp_packet_id
    packets: BTreeMap<(usize, bool), (usize, M)>,
}

impl<M> ReadyTcpPacketFilter<M> {
    pub fn new() -> Self {
        Self {
            packets: Default::default(),
        }
    }

    pub fn add(&mut self, packet: &TcpPacketKind, meta: M) {
        let entry = self.packets.entry((packet.stream, packet.dir));
        match entry {
            Entry::Occupied(mut e) => {
                if packet.tcp_packet_id < e.get().0 {
                    e.insert((packet.tcp_packet_id, meta));
                }
            }
            Entry::Vacant(e) => {
                e.insert((packet.tcp_packet_id, meta));
            }
        }
    }

    pub fn ready_packets(&self) -> impl Iterator<Item = &(usize, M)> + '_ {
        self.packets.values()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{ReadyTcpPacketFilter, TcpPacketKind};

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut filter = ReadyTcpPacketFilter::new();
        filter.add(&TcpPacketKind::new(0, 0, false), ());
        filter.add(&TcpPacketKind::new(1, 1, false), ());
        let mut packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        packets.sort();
        assert_eq!(packets, [0, 1]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn select_packet_with_min_id() {
        let mut filter = ReadyTcpPacketFilter::new();
        filter.add(&TcpPacketKind::new(2, 0, false), ());
        filter.add(&TcpPacketKind::new(1, 0, false), ());
        filter.add(&TcpPacketKind::new(3, 0, false), ());
        let packets = filter
            .ready_packets()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(packets, [1]);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn different_dirrections() {
        let mut filter = ReadyTcpPacketFilter::new();
        filter.add(&TcpPacketKind::new(2, 0, false), ());
        filter.add(&TcpPacketKind::new(1, 0, true), ());
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
        let mut filter = ReadyTcpPacketFilter::new();
        let mut add = |id, stream, dir| {
            filter.add(&TcpPacketKind::new(id, stream, dir), ());
        };
        add(1, 0, false);
        add(2, 0, false);
        add(4, 0, true);
        add(6, 1, true);
        add(7, 2, false);
        add(5, 0, true);
        add(3, 0, true);
        add(0, 0, false);
        add(10, 2, false);
        add(8, 2, false);
        add(9, 1, true);
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
            let mut filter = ReadyTcpPacketFilter::new();
            let mut add = |id, stream, dir| {
                filter.add(&TcpPacketKind::new(id, stream, dir), ());
            };
            add(1, 0, false);
            add(2, 0, false);
            add(4, 0, true);
            add(6, 1, true);
            add(7, 2, false);
            add(5, 0, true);
            add(3, 0, true);
            add(0, 0, false);
            add(10, 2, false);
            add(8, 2, false);
            add(9, 1, true);
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
        let mut filter = ReadyTcpPacketFilter::new();
        filter.add(&TcpPacketKind::new(1, 0, false), 123);
        filter.add(&TcpPacketKind::new(0, 0, false), 321);
        let packets = filter.ready_packets().collect::<Vec<_>>();
        assert_eq!(packets, [&(0, 321)]);
    }
}
