use std::hash::{DefaultHasher, Hash, Hasher};

////////////////////////////////////////////////////////////////////////////////

pub fn hash_set(elems: impl Iterator<Item = u64>) -> u64 {
    let mut result = 0;
    elems.for_each(|e| result ^= e);
    result
}

////////////////////////////////////////////////////////////////////////////////

pub fn hash_multiset(elems: impl Iterator<Item = u64>) -> u64 {
    let mut v = elems.collect::<Vec<_>>();
    v.sort();
    hash_list(v.iter().copied())
}

////////////////////////////////////////////////////////////////////////////////

pub fn hash_list(elems: impl Iterator<Item = u64>) -> u64 {
    let mut hasher = DefaultHasher::new();
    elems.for_each(|e| e.hash(&mut hasher));
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::{hash_list, hash_multiset, hash_set};

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn set() {
        assert_eq!(hash_set(0..5), hash_set(vec![0, 1, 3, 2, 4].into_iter()));
        assert!(hash_set(0..5) != 0);
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn multiset() {
        assert_eq!(
            hash_multiset(0..5),
            hash_multiset([0, 1, 3, 2, 4].into_iter())
        );
        assert!(hash_multiset([0, 1].into_iter()) != hash_multiset([0, 1, 1].into_iter()));
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn list() {
        assert!(hash_list([1, 2, 3].into_iter()) != hash_list([1, 3, 2].into_iter()));
    }
}
