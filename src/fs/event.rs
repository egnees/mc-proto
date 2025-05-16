use std::{fmt::Display, time::Duration};

use crate::{
    sim::log::{
        CreateFileRequested, DeleteFileRequested, OpenFileRequested, ReadFileCompleted,
        ReadFileInitiated, WriteFileCompleted, WriteFileInitiated,
    },
    Address, LogEntry,
};

use super::error::FsError;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Hash)]
pub enum FsEventKind {
    Create {
        file: String,
    },
    Delete {
        file: String,
    },
    Open {
        file: String,
    },
    Read {
        file: String,
        offset: usize,
        len: usize,
    },
    Write {
        file: String,
        offset: usize,
        len: usize,
    },
}

impl Display for FsEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsEventKind::Create { file } => write!(f, "Create file: {file}"),
            FsEventKind::Delete { file } => write!(f, "Delete file: {file}"),
            FsEventKind::Open { file } => write!(f, "Open file {file}"),
            FsEventKind::Read { file, offset, len } => {
                write!(f, "Read file {file}[{}..{}]", *offset, *offset + *len)
            }
            FsEventKind::Write { file, offset, len } => {
                write!(f, "Write file {file}[{}..{}]", *offset, *offset + *len)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub type FsEventOutcome = Result<(), FsError>;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct FsEvent {
    pub initiated_by: Address,
    pub kind: FsEventKind,
    pub outcome: FsEventOutcome,
}

////////////////////////////////////////////////////////////////////////////////

impl FsEvent {
    pub fn make_log_entry_on_init(self, time: Duration) -> LogEntry {
        match self.kind {
            FsEventKind::Create { file } => {
                let entry = CreateFileRequested {
                    time,
                    proc: self.initiated_by,
                    file,
                    outcome: self.outcome,
                };
                LogEntry::CreateFileRequested(entry)
            }
            FsEventKind::Delete { file } => {
                let entry = DeleteFileRequested {
                    time,
                    proc: self.initiated_by,
                    file,
                    outcome: self.outcome,
                };
                LogEntry::DeleteFileRequested(entry)
            }
            FsEventKind::Open { file } => {
                let entry = OpenFileRequested {
                    time,
                    proc: self.initiated_by,
                    file,
                    outcome: self.outcome,
                };
                LogEntry::OpenFileRequested(entry)
            }
            FsEventKind::Read { file, .. } => {
                let entry = ReadFileInitiated {
                    time,
                    proc: self.initiated_by,
                    file,
                };
                LogEntry::ReadFileInitiated(entry)
            }
            FsEventKind::Write { file, .. } => {
                let entry = WriteFileInitiated {
                    time,
                    proc: self.initiated_by,
                    file,
                };
                LogEntry::WriteFileInitiated(entry)
            }
        }
    }

    pub fn make_log_entry_on_complete(self, time: Duration) -> LogEntry {
        match self.kind {
            FsEventKind::Read { file, .. } => {
                let entry = ReadFileCompleted {
                    time,
                    proc: self.initiated_by,
                    file,
                    outcome: self.outcome,
                };
                LogEntry::ReadFileCompleted(entry)
            }
            FsEventKind::Write { file, .. } => {
                let entry = WriteFileCompleted {
                    time,
                    proc: self.initiated_by,
                    file,
                    outcome: self.outcome,
                };
                LogEntry::WriteFileCompleted(entry)
            }
            _ => unreachable!(),
        }
    }
}
