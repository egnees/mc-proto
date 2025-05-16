use crate::tracker::EventTracker;

use super::engine::Engine;

////////////////////////////////////////////////////////////////////////////////

fn explore(
    t: impl EventTracker<i64>,
    depth: usize,
    max_depth: usize,
    engine: &mut impl Engine,
) -> usize {
    if depth == max_depth {
        return 1;
    }
    let mut cnt = 0;
    t.next_events().for_each(|(e, mut t)| {
        cnt += 1;
        engine.add_events(e, &mut t);
        cnt += explore(t, depth + 1, max_depth, engine);
    });
    if cnt == 0 {
        assert_eq!(t.pending_events().count(), 0);
    }
    cnt
}

pub fn make_exploration(
    max_depth: usize,
    mut engine: impl Engine,
    mut t: impl EventTracker<i64>,
) -> usize {
    engine.add_events(0, &mut t);
    explore(t, 1, max_depth, &mut engine)
}

////////////////////////////////////////////////////////////////////////////////

const fn ranges0() -> [(i64, i64); 4] {
    [(1, 1), (1, 2), (3, 4), (5, 10)]
}

const fn ranges1() -> [(i64, i64); 4] {
    [(1, 1), (100, 2), (300, 4), (5, 10)]
}

const fn ranges2() -> [(i64, i64); 4] {
    [(100, 100), (10, 12), (300, 4), (5, 100)]
}

pub const RANGES: [[(i64, i64); 4]; 3] = [ranges0(), ranges1(), ranges2()];
