use crate::fs;

////////////////////////////////////////////////////////////////////////////////

pub struct FsEventKind {
    #[allow(unused)]
    pub kind: fs::event::FsEventKind,
    pub outcome: fs::event::FsEventOutcome,
}
