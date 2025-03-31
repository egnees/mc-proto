//! Definition of segment in the one dimention space.

////////////////////////////////////////////////////////////////////////////////
pub trait Endpoint: Ord + Copy + Default {}

////////////////////////////////////////////////////////////////////////////////

impl<T> Endpoint for T where T: Ord + Copy + Default {}

////////////////////////////////////////////////////////////////////////////////

/// Represents segment with custom type of endpoints.
/// Segments can be customized with [tags](Segment::tag).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Segment<T>
where
    T: Endpoint,
{
    /// Start of the segment.
    pub from: T,

    /// End of the segment.
    pub to: T,

    /// Custom tag.
    pub tag: usize,
}

////////////////////////////////////////////////////////////////////////////////

impl<T> Segment<T>
where
    T: Endpoint,
{
    pub fn new(from: T, to: T, tag: usize) -> Self {
        Self { from, to, tag }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ord() {
        let a = Segment::new(1, 2, 0);
        let b = Segment::new(1, 2, 1);
        let c = Segment::new(1, 3, 0);
        let d = Segment::new(2, 2, 0);
        let segments = [a, b, c, d];
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(segments[i] < segments[j]);
            }
        }
    }
}
