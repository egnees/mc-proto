use super::segment::{Endpoint, Segment};

////////////////////////////////////////////////////////////////////////////////

/// Allows to track ready list of segments in one dimentional space.
/// Ready list is composed of segments which have not segments
/// strictly on the right from them (r1 < l2).
///
/// Minimal endpoint of segments from tracking set must increase
/// with adding new segments.
///
/// Tags of segments must be unique.
pub trait Tracker<T>
where
    T: Endpoint,
{
    /// Allows to add segment with provided `tag`,
    /// which starts in `from` and ends in `to`.
    ///
    /// Left endpoints of adding segments must increase.
    fn add(&mut self, from: T, to: T, tag: usize);

    /// Allows to remove segment with provided tag.
    fn remove_with_tag(&mut self, tag: usize) -> Option<Segment<T>>;

    /// Allows to remove i-th segment from ready list.
    fn remove_ready(&mut self, i: usize) -> Option<Segment<T>>;

    /// Allows to get size of ready set.
    fn ready_count(&self) -> usize;

    /// Allows to get i-th ready segment.
    fn ready(&self, i: usize) -> Option<&Segment<T>>;
}
