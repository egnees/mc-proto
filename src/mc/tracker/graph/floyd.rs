use std::ops::Add;

use super::matrix::MaxMatrix;

////////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
pub fn floyd_closure<T: Ord + Add<Output = T> + Copy>(m: &mut MaxMatrix<T>) {
    let n = m.size();
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if let Some(w) = m.sum_edge(i, k, j) {
                    m.add_edge(i, j, w);
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::mc::tracker::graph::matrix::MaxMatrix;

    use super::floyd_closure;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut m = MaxMatrix::new(5);

        // from 0
        m.add_edge(0, 1, 4);
        m.add_edge(0, 2, -5);

        // from 1
        m.add_edge(1, 2, -8);
        m.add_edge(1, 3, 2);

        // from 2
        m.add_edge(2, 3, 3);
        m.add_edge(2, 4, 1);

        // from 3
        m.add_edge(3, 4, -2);

        // from 4
        m.add_edge(4, 2, -1);
        m.add_edge(4, 0, -100);

        floyd_closure(&mut m);

        assert_eq!(m.edge(4, 2), Some(-1));
        assert_eq!(m.edge(4, 3), Some(2));
        assert_eq!(m.edge(4, 0), Some(-100));
        assert_eq!(m.edge(2, 4), Some(1));
        assert_eq!(m.edge(0, 2), Some(3));
        assert_eq!(m.edge(2, 3), Some(3));
        assert_eq!(m.edge(3, 4), Some(-2));
        assert_eq!(m.edge(2, 1), Some(-95));
        assert_eq!(m.edge(3, 1), Some(-98));

        for i in 0..5 {
            assert_eq!(m.edge(i, i), Some(0));
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn no_paths() {
        let mut m = MaxMatrix::new(3);

        m.add_edge(1, 2, 0);
        m.add_edge(0, 2, 0);

        floyd_closure(&mut m);

        assert_eq!(m.edge(0, 2), Some(0));
        assert_eq!(m.edge(1, 2), Some(0));

        for i in 0..3 {
            assert_eq!(m.edge(i, i), Some(0));
        }

        assert!(m.edge(0, 1).is_none());

        assert!(m.edge(1, 0).is_none());

        assert!(m.edge(2, 0).is_none());
        assert!(m.edge(2, 1).is_none());
    }
}
