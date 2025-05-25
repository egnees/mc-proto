use rstest::rstest;

use crate::mc::tracker::{
    floyd::FloydEventTracker,
    tests::{
        engine::{FixedTimeEngine, RandomTimeEngine},
        stress::{make_exploration, RANGES},
    },
};

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(3, 3)]
#[case(4, 4)]
#[case(10, 2)]
#[case(10, 3)]
#[case(7, 5)]
fn stress_with_random_time_engine(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (random time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        RandomTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(3, 3)]
#[case(4, 4)]
#[case(10, 2)]
#[case(10, 3)]
#[case(7, 5)]
fn stress_with_fixed_time_engine(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (fixed time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        FixedTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(15, 2)]
#[case(13, 3)]
#[case(12, 4)]
#[cfg(not(debug_assertions))]
fn stress_random_test_engine_deep(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (fixed time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        RandomTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(15, 2)]
#[case(13, 3)]
#[cfg(not(debug_assertions))]
fn stress_fixed_test_engine_deep(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (fixed time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        FixedTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(7, 7)]
#[case(8, 6)]
#[case(9, 5)]
#[cfg(not(debug_assertions))]
fn stress_random_test_engine_wide(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (fixed time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        RandomTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}

////////////////////////////////////////////////////////////////////////////////

#[rstest]
#[case(7, 7)]
#[case(8, 6)]
#[cfg(not(debug_assertions))]
fn stress_fixed_test_engine_wide(
    #[case] max_depth: usize,
    #[case] max_children_events: usize,
    #[values(0, 1, 2)] range: usize,
    #[values(123, 321, 0, 15)] seed: u64,
) {
    println!("Make exploration (fixed time engine): max_depth={max_depth}, max_children_events={max_children_events}, range: {range}, seed: {seed}");
    let explored = make_exploration(
        max_depth,
        FixedTimeEngine::new(seed, max_children_events, RANGES[range].into_iter()),
        FloydEventTracker::default(),
    );
    println!("Explored: {explored}");
}
