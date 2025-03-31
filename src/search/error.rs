use std::fmt::{Debug, Display};

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

impl Display for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Liveness violation: ")?;
        if self.trace.is_some() {
            writeln!(f, "found terminal state not achieving goal.")?;
            writeln!(f, "========= TRACE =========")?;
            write!(f, "{}", self.trace.as_ref().unwrap())?;
            writeln!(f, "========= LOG =========")?;
            write!(f, "{}", self.log.as_ref().unwrap())?;
        } else {
            writeln!(f, "not found states achieving goal.")?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum SearchError {
    InvariantViolation(InvariantViolation),
    LivenessViolation(LivenessViolation),
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
        }
    }
}
