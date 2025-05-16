use std::ops::Add;

#[derive(Clone)]
pub struct MaxMatrix<T> {
    d: Vec<Vec<Option<T>>>,
}

impl<T: Default + Copy> MaxMatrix<T> {
    pub fn new(n: usize) -> Self {
        let d = (0..n)
            .map(|i| {
                let mut v = vec![None; n];
                v[i] = Some(T::default());
                v
            })
            .collect();
        Self { d }
    }

    pub fn add_vertex(&mut self) {
        self.d.iter_mut().for_each(|v| v.push(None));
        {
            let mut v = vec![None; self.d.len() + 1];
            *v.last_mut().unwrap() = Some(T::default());
            self.d.push(v);
        }
    }
}

impl<T> MaxMatrix<T> {
    pub fn size(&self) -> usize {
        self.d.len()
    }
}

impl<T: Copy + Ord + Add<Output = T>> MaxMatrix<T> {
    pub fn edge(&self, from: usize, to: usize) -> Option<T> {
        self.d[from][to]
    }

    pub fn edge_mut(&mut self, from: usize, to: usize) -> &mut Option<T> {
        &mut self.d[from][to]
    }

    pub fn add_edge(&mut self, from: usize, to: usize, w: T) -> T {
        let cur = self.edge_mut(from, to).get_or_insert(w);
        *cur = (*cur).max(w);
        *cur
    }

    pub fn sum_edge(&self, i: usize, j: usize, k: usize) -> Option<T> {
        let a = self.edge(i, j)?;
        let b = self.edge(j, k)?;
        Some(a + b)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::MaxMatrix;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut m = MaxMatrix::new(3);
        m.add_edge(0, 1, 5);
        m.add_edge(0, 1, 6);
        assert_eq!(m.edge(0, 1).unwrap(), 6);
        assert_eq!(m.edge(1, 0), None);
        m.add_edge(0, 1, 5);
        assert_eq!(m.edge(0, 1).unwrap(), 6);
        m.add_edge(1, 2, 3);
        assert_eq!(m.sum_edge(0, 1, 2).unwrap(), 6 + 3);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn add_vertex() {
        let mut m = MaxMatrix::new(1);

        assert_eq!(m.edge(0, 0), Some(0));

        // add vertex 1
        m.add_vertex();
        assert_eq!(m.edge(1, 1), Some(0));
        assert_eq!(m.edge(0, 1), None);
        assert_eq!(m.edge(1, 0), None);

        m.add_edge(0, 1, 5);
        m.add_edge(0, 1, 6);

        assert_eq!(m.edge(0, 1), Some(6));

        m.add_vertex();
        m.add_edge(1, 2, 3);

        assert_eq!(m.sum_edge(0, 1, 2).unwrap(), 6 + 3);
    }
}
