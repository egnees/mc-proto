use std::ops::Add;

use super::{floyd::floyd_closure, matrix::MaxMatrix, moore::MooreGraph};

pub trait Graph<T>: Clone {
    fn add_vertex(&mut self) -> usize;
    fn add_edge(&mut self, from: usize, to: usize, w: T);
    fn max_dist(&self, from: usize, to: usize) -> Option<T>;
    fn size(&self) -> usize;
}

////////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
#[derive(Clone)]
pub struct GraphFloyd<T> {
    d: MaxMatrix<T>,
}

impl<T: Copy + Ord + Add<Output = T> + Default> Graph<T> for GraphFloyd<T> {
    fn add_edge(&mut self, from: usize, to: usize, w: T) {
        self.d.add_edge(from, to, w);
    }

    fn max_dist(&self, from: usize, to: usize) -> Option<T> {
        let mut d = self.d.clone();
        floyd_closure(&mut d);
        d.edge(from, to)
    }

    fn add_vertex(&mut self) -> usize {
        let v = self.d.size();
        self.d.add_vertex();
        v
    }

    fn size(&self) -> usize {
        self.d.size()
    }
}

impl<T: Default + Copy> GraphFloyd<T> {
    pub fn new(n: usize) -> Self {
        Self {
            d: MaxMatrix::new(n),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct GraphFloydSmart<T> {
    d: MaxMatrix<T>,
}

impl<T: Copy + Ord + Add<Output = T> + Default> Graph<T> for GraphFloydSmart<T> {
    fn add_vertex(&mut self) -> usize {
        let v = self.d.size();
        self.d.add_vertex();
        v
    }

    fn add_edge(&mut self, from: usize, to: usize, w: T) {
        self.d.add_edge(from, to, w);

        let n = self.d.size();

        for i in 0..n {
            for j in 0..n {
                if let Some(w) = (|| {
                    let i_from = self.d.edge(i, from)?;
                    let to_j = self.d.edge(to, j)?;
                    Some(i_from + w + to_j)
                })() {
                    self.d.add_edge(i, j, w);
                }
            }
        }
    }

    fn max_dist(&self, from: usize, to: usize) -> Option<T> {
        self.d.edge(from, to)
    }

    fn size(&self) -> usize {
        self.d.size()
    }
}

impl<T: Default + Copy> GraphFloydSmart<T> {
    pub fn new(n: usize) -> Self {
        Self {
            d: MaxMatrix::new(n),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    ////////////////////////////////////////////////////////////////////////////////

    use rand::SeedableRng;
    use rand::{rngs::StdRng, Rng};

    use crate::mc::tracker::graph::graph_floyd::{Graph, GraphFloyd, GraphFloydSmart};

    #[test]
    fn graph_floyd() {
        let mut g = GraphFloyd::new(0);

        // 0
        g.add_vertex();

        // 1
        g.add_vertex();

        // 2
        g.add_vertex();

        // from 0
        g.add_edge(0, 1, 4);
        g.add_edge(0, 2, -5);

        // 3
        g.add_vertex();

        // from 1
        g.add_edge(1, 2, -8);
        g.add_edge(1, 3, 2);

        // 4
        g.add_vertex();

        // from 2
        g.add_edge(2, 3, 3);
        g.add_edge(2, 4, 1);

        // from 3
        g.add_edge(3, 4, -2);

        // from 4
        g.add_edge(4, 2, -1);
        g.add_edge(4, 0, -100);

        assert_eq!(g.max_dist(4, 2), Some(-1));
        assert_eq!(g.max_dist(4, 3), Some(2));
        assert_eq!(g.max_dist(4, 0), Some(-100));
        assert_eq!(g.max_dist(2, 4), Some(1));
        assert_eq!(g.max_dist(0, 2), Some(3));
        assert_eq!(g.max_dist(2, 3), Some(3));
        assert_eq!(g.max_dist(3, 4), Some(-2));
        assert_eq!(g.max_dist(2, 1), Some(-95));
        assert_eq!(g.max_dist(3, 1), Some(-98));

        for i in 0..5 {
            assert_eq!(g.max_dist(i, i), Some(0));
        }

        assert_eq!(g.size(), 5);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn graph_floyd_smart() {
        let mut g = GraphFloydSmart::new(0);

        // 0
        g.add_vertex();

        // 1
        g.add_vertex();

        // 2
        g.add_vertex();

        // from 0
        g.add_edge(0, 1, 4);
        g.add_edge(0, 2, -5);

        // 3
        g.add_vertex();

        // from 1
        g.add_edge(1, 2, -8);
        g.add_edge(1, 3, 2);

        // 4
        g.add_vertex();

        // from 2
        g.add_edge(2, 3, 3);
        g.add_edge(2, 4, 1);

        // from 3
        g.add_edge(3, 4, -2);

        // from 4
        g.add_edge(4, 2, -1);
        g.add_edge(4, 0, -100);

        assert_eq!(g.max_dist(4, 2), Some(-1));
        assert_eq!(g.max_dist(4, 3), Some(2));
        assert_eq!(g.max_dist(4, 0), Some(-100));
        assert_eq!(g.max_dist(2, 4), Some(1));
        assert_eq!(g.max_dist(0, 2), Some(3));
        assert_eq!(g.max_dist(2, 3), Some(3));
        assert_eq!(g.max_dist(3, 4), Some(-2));
        assert_eq!(g.max_dist(2, 1), Some(-95));
        assert_eq!(g.max_dist(3, 1), Some(-98));

        for i in 0..5 {
            assert_eq!(g.max_dist(i, i), Some(0));
        }

        assert_eq!(g.size(), 5);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn stress() {
        let n = 20;

        let mut correct = GraphFloydSmart::new(n);
        let mut test = GraphFloydSmart::new(n);

        let mut rng = StdRng::seed_from_u64(123);

        for _ in 0..150 {
            let i = rng.random_range(0..n);
            let j = rng.random_range(0..n);
            let from = -100i64;
            let to = 10i64;
            let w = rng.random_range(from..to);
            correct.add_edge(i, j, w);
            test.add_edge(i, j, w);

            for i in 0..n {
                for j in 0..n {
                    assert_eq!(correct.max_dist(i, j), test.max_dist(i, j));
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn raft_like_example_dummy() {
        let mut g = GraphFloyd::new(1);
        let v0 = 0;
        let v1 = g.add_vertex();

        g.add_edge(v0, v1, 250);
        g.add_edge(v1, v0, -750);

        let v2 = g.add_vertex();
        g.add_edge(v0, v2, 250);
        g.add_edge(v2, v0, -750);

        // 1 happen
        g.add_edge(v1, v2, 0);

        let v3 = g.add_vertex();
        g.add_edge(v1, v3, 1);
        g.add_edge(v3, v1, -5);

        let v4 = g.add_vertex();
        g.add_edge(v1, v4, 1);
        g.add_edge(v4, v1, -5);

        // 2 happen
        g.add_edge(v2, v3, 0);
        g.add_edge(v2, v4, 0);

        let v5 = g.add_vertex();
        g.add_edge(v2, v5, 250);
        g.add_edge(v5, v2, -750);

        // 3 happen
        g.add_edge(v3, v4, 0);
        g.add_edge(v3, v5, 0);

        {
            // check v4 can happen
            let mut g = g.clone();
            g.add_edge(v4, v5, 0);

            for i in 0..6 {
                assert!(g.max_dist(0, i).unwrap() <= -g.max_dist(i, 0).unwrap());
            }
        }

        {
            // check v5 can not happen
            let mut g = g.clone();
            g.add_edge(v5, v4, 0);

            let mut ok = true;
            for i in 0..6 {
                if g.max_dist(i, i).unwrap() != 0 {
                    ok = false;
                }
            }

            assert!(!ok);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn raft_like_example() {
        let mut g = GraphFloydSmart::new(1);
        let v0 = 0;
        let v1 = g.add_vertex();

        g.add_edge(v0, v1, 250);
        g.add_edge(v1, v0, -750);

        let v2 = g.add_vertex();
        g.add_edge(v0, v2, 250);
        g.add_edge(v2, v0, -750);

        // 1 happen
        g.add_edge(v1, v2, 0);

        let v3 = g.add_vertex();
        g.add_edge(v1, v3, 1);
        g.add_edge(v3, v1, -5);

        let v4 = g.add_vertex();
        g.add_edge(v1, v4, 1);
        g.add_edge(v4, v1, -5);

        // 2 happen
        g.add_edge(v2, v3, 0);
        g.add_edge(v2, v4, 0);

        let v5 = g.add_vertex();
        g.add_edge(v2, v5, 250);
        g.add_edge(v5, v2, -750);

        // 3 happen
        g.add_edge(v3, v4, 0);
        g.add_edge(v3, v5, 0);

        {
            // check v4 can happen
            let mut g = g.clone();
            g.add_edge(v4, v5, 0);

            for i in 0..6 {
                assert!(g.max_dist(0, i).unwrap() <= -g.max_dist(i, 0).unwrap());
            }
        }

        {
            // check v5 can not happen
            let mut g = g.clone();
            g.add_edge(v5, v4, 0);
            let mut ok = true;
            for i in 0..6 {
                if g.max_dist(i, i).unwrap() != 0 {
                    ok = false;
                }
            }
            assert!(!ok);
        }
    }
}
