use std::fmt::{Debug, Display};

use crate::{sim::log::Log, HashType};

use super::{log::SearchLog, state::StateTrace};

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
        write!(f, "{}", self)
    }
}

impl Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Invariant violation: {}.", self.report)?;
        writeln!(f, "======== TRACE ========")?;
        write!(f, "{}", self.trace)?;
        writeln!(f, "========= LOG =========")?;
        write!(f, "{}", self.log)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct LivenessViolation {
    pub trace: StateTrace,
    pub log: Log,
    pub report: String,
}

impl LivenessViolation {
    pub fn new(trace: StateTrace, log: Log, report: String) -> Self {
        Self { trace, log, report }
    }
}

impl Debug for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for LivenessViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Liveness violation: ")?;
        writeln!(f, "found terminal state which not achieves goal.")?;
        writeln!(f, "Reason: {}.", self.report)?;
        writeln!(f, "======== TRACE ========")?;
        write!(f, "{}", self.trace)?;
        writeln!(f, "========= LOG =========")?;
        write!(f, "{}", self.log)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct AllPruned {
    pub last_trace: StateTrace,
    pub last_log: Log,
}

impl AllPruned {
    pub fn new(last_trace: StateTrace, last_log: Log) -> Self {
        Self {
            last_trace,
            last_log,
        }
    }
}

impl Debug for AllPruned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for AllPruned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Pruned all states and did not find state achieving goal."
        )?;
        writeln!(f, "======== LAST PRUNED TRACE ========")?;
        write!(f, "{}", self.last_trace)?;
        writeln!(f, "========= LAST PRUNED LOG =========")?;
        write!(f, "{}", self.last_log)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Cycled {
    pub trace: StateTrace,
    pub log: Log,
    pub hash: HashType,
}

impl Cycled {
    pub fn new(trace: StateTrace, log: Log, hash: HashType) -> Self {
        Self { trace, log, hash }
    }
}

impl Debug for Cycled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Cycled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found cycle which is impossible to get out of.")?;
        writeln!(f, "======== TRACE ========")?;
        write!(f, "{}", self.trace)?;
        writeln!(f, "========= LOG =========")?;
        writeln!(f, "{}", self.log)?;
        write!(f, "State hash: {}", self.hash)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum SearchErrorKind {
    InvariantViolation(InvariantViolation),
    LivenessViolation(LivenessViolation),
    AllPruned(AllPruned),
    ProcessPanic(ProcessPanic),
    Cycled(Cycled),
}

impl Display for SearchErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchErrorKind::InvariantViolation(invariant_violation) => {
                write!(f, "{}", invariant_violation)
            }
            SearchErrorKind::LivenessViolation(liveness_violation) => {
                write!(f, "{}", liveness_violation)
            }
            SearchErrorKind::ProcessPanic(p) => write!(f, "{}", p),
            SearchErrorKind::AllPruned(err) => write!(f, "{}", err),
            SearchErrorKind::Cycled(cycled) => write!(f, "{}", cycled),
        }
    }
}

impl Debug for SearchErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct SearchError {
    pub kind: SearchErrorKind,
    pub log: SearchLog,
}

impl SearchError {
    pub fn new(kind: SearchErrorKind, log: &SearchLog) -> Self {
        Self {
            kind,
            log: log.clone(),
        }
    }
}

impl Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nSearch error:")?;
        writeln!(f, "{}", self.kind)?;
        writeln!(f, "\nSearch log:")?;
        write!(f, "{}", self.log)
    }
}

impl Debug for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
