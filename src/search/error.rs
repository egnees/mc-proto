use std::fmt::{Debug, Display};

use crate::sim::log::Log;

use super::state::StateTrace;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessPanic {
    pub trace: Option<StateTrace>,
    pub log: Log,
}

impl Display for ProcessPanic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Process panic.")?;
        if let Some(trace) = self.trace.as_ref() {
            writeln!(f, "========= TRACE =========")?;
            write!(f, "{}", trace)?;
        }
        writeln!(f, "========= LOG =========")?;
        write!(f, "{}", self.log)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct InvariantViolation {
    pub trace: StateTrace,
    pub log: Log,
    pub report: String,
}

impl Debug for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)
    }
}

impl Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Invariant violation: {}.", self.report)?;
        writeln!(f, "========= TRACE =========")?;
        write!(f, "{}", self.trace)?;
        writeln!(f, "========= LOG =========")?;
        write!(f, "{}", self.log)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct LivenessViolation {
    pub trace: Option<StateTrace>,
    pub log: Option<Log>,
}

impl LivenessViolation {
    pub fn no_one() -> Self {
        Self {
            trace: None,
            log: None,
        }
    }

    pub fn this_one(trace: StateTrace, log: Log) -> Self {
        Self {
            trace: Some(trace),
            log: Some(log),
        }
    }
}

impl Debug for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)
    }
}

impl Display for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Liveness violation: ")?;
        if self.trace.is_some() {
            writeln!(f, "found terminal state which not achieves goal.")?;
            writeln!(f, "========= TRACE =========")?;
            write!(f, "{}", self.trace.as_ref().unwrap())?;
            writeln!(f, "========= LOG =========")?;
            write!(f, "{}", self.log.as_ref().unwrap())?;
        } else {
            writeln!(f, "not found states which achieve goal.")?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum SearchError {
    InvariantViolation(InvariantViolation),
    LivenessViolation(LivenessViolation),
    ProcessPanic(ProcessPanic),
}

impl Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::InvariantViolation(invariant_violation) => {
                writeln!(f, "{}", invariant_violation)
            }
            SearchError::LivenessViolation(liveness_violation) => {
                writeln!(f, "{}", liveness_violation)
            }
            SearchError::ProcessPanic(p) => writeln!(f, "{}", p),
        }
    }
}

impl Debug for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)
    }
}
