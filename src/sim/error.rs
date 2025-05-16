#[derive(Clone, Debug)]
pub enum Error {
    AlreadyExists,
    NotFound,
    NegativeDelay,
    NegativeTime,
    IncorrectRange,
    FsAlreadySetup,
    FsNotAvailable,
    NodeUnavailable,
}
