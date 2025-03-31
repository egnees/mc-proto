use std::fmt::Debug;

use crate::system::log::Log;

use super::trace::Trace;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct InvariantViolation {
    pub trace: Trace,
    pub log: Log,
    pub report: String,
}

impl Debug for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InvariantViolation")
            .field("log", &self.log)
            .field("report", &self.report)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct LivenessViolation {
    pub trace: Option<Trace>,
    pub log: Option<Log>,
}

impl LivenessViolation {
    pub fn no_one() -> Self {
        Self {
            trace: None,
            log: None,
        }
    }

    pub fn this_one(trace: Trace, log: Log) -> Self {
        Self {
            trace: Some(trace),
            log: Some(log),
        }
    }
}

impl Debug for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivenessViolation")
            .field("log", &self.log)
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum SearchError {
    InvariantViolation(InvariantViolation),
    LivenessViolation(LivenessViolation),
}
