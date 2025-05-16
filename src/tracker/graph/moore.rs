use std::{collections::VecDeque, ops::Add};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct MooreGraph<T> {
    dist: Vec<Option<T>>,
    in_q: Vec<bool>,
    g: Vec<Vec<(usize, T)>>,
}

impl<T: Default> Default for MooreGraph<T> {
    fn default() -> Self {
        Self {
            dist: vec![Some(T::default())],
            in_q: vec![false],
            g: vec![Vec::default()],
        }
    }
}

impl<T: Default + Copy + Ord + Add<Output = T>> MooreGraph<T> {
    // check if d[to] < d[from] + w
    // return Some(d[from] + w) in that case
    fn check(&mut self, from: usize, to: usize, w: T) -> Option<T> {
        let w1 = self.dist[from]? + w;
        let dist_to = self.dist[to].get_or_insert(w1);
        if *dist_to < w1 {
            *dist_to = w1;
            Some(*dist_to)
        } else {
            None
        }
    }

    pub fn add_edge(mut self, from: usize, to: usize, w: T) -> Option<Self> {
        self.g[from].push((to, w));
        if self.check(from, to, w).is_none() {
            return Some(self);
        }
        let mut q = VecDeque::new();
        q.push_back(to);
        self.in_q[to] = true;
        while let Some(v) = q.pop_front() {
            self.in_q[v] = false;
            if v == from {
                // found positive cycle
                return None;
            }
            for i in 0..self.g[v].len() {
                let (to, w) = self.g[v][i];
                let check_result = self.check(v, to, w);
                if check_result.is_some() {
                    self.in_q[to] = true;
                    q.push_back(to);
                }
            }
        }
        Some(self)
    }

    pub fn add_vertex(&mut self) -> usize {
        let v = self.dist.len();
        self.dist.push(None);
        self.g.push(Vec::default());
        self.in_q.push(false);
        v
    }

    pub fn dist(&self, to: usize) -> Option<T> {
        self.dist[to]
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use rstest::rstest;

    use crate::tracker::graph::{graph::GraphFloyd, Graph};

    use super::MooreGraph;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut g = MooreGraph::default();
        let v1 = g.add_vertex();
        let g = g.add_edge(0, v1, 1).unwrap();
        let result = g.add_edge(v1, 0, 0);
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn random() {
        let mut g = MooreGraph::default();

        // 0
        g.add_vertex();

        // 1
        g.add_vertex();

        // 2
        g.add_vertex();

        // from 0
        let g = g.add_edge(0, 1, 4).unwrap();
        let mut g = g.add_edge(0, 2, -5).unwrap();

        // 3
        g.add_vertex();

        // from 1
        let g = g.add_edge(1, 2, -8).unwrap();
        let mut g = g.add_edge(1, 3, 2).unwrap();

        // 4
        g.add_vertex();

        // from 2
        let g = g.add_edge(2, 3, 3).unwrap();
        let g = g.add_edge(2, 4, 1).unwrap();

        // from 3
        let g = g.add_edge(3, 4, -2).unwrap();

        // from 4
        let g = g.add_edge(4, 2, -1).unwrap();
        let g = g.add_edge(4, 0, -100).unwrap();

        let result = g.add_edge(0, 4, 101);
        assert!(result.is_none());
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn check_cycle_floyd(g: &GraphFloyd<i64>) -> bool {
        for v in 0..g.size() {
            if g.max_dist(v, v).unwrap() > 0 {
                return true;
            }
        }
        false
    }

    #[rstest]
    fn stress_vs_floyd(
        #[values(1, 2, 3, 123, 321)] seed: u64,
        #[values(8, 10, 12, 15, 20)] n: usize,
    ) {
        let mut moore = MooreGraph::default();
        for _ in 1..n {
            moore.add_vertex();
        }
        let mut floyd = GraphFloyd::new(n);

        let mut rng = StdRng::seed_from_u64(seed);
        for v in 1..n {
            let w = rng.random_range(-1000..100);
            moore = moore.add_edge(0, v, w).unwrap();
            floyd.add_edge(0, v, w);
            assert!(!check_cycle_floyd(&floyd));
        }
        for iter in 1..=100000 {
            let from = rng.random_range(0..n);
            let to = rng.random_range(0..n);
            let w = rng.random_range(-1000..100);
            floyd.add_edge(from, to, w);
            let x = moore.add_edge(from, to, w);
            let cycle_moore = x.is_none();
            let cycle_floyd = check_cycle_floyd(&floyd);
            assert_eq!(cycle_moore, cycle_floyd);
            if cycle_moore {
                println!("cycle found on iter={iter}");
                break;
            }
            moore = x.unwrap();
        }
    }
}
