//! Errors which can in the system model during the search.

use std::fmt::{Debug, Display};

use crate::{model::log::Log, HashType};

use super::{log::SearchLog, state::StateTrace};

////////////////////////////////////////////////////////////////////////////////

/// Some process panic.
#[derive(Clone)]
pub struct ProcessPanic {
    /// Trace which leads to the state, in which process panic.
    /// Allows to rerun system in the debug mode on the failing event sequence.
    pub trace: Option<StateTrace>,

    /// Log of system events
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

/// Search invariant violation (see [`crate::mc::InvariantFn`]).
#[derive(Clone)]
pub struct InvariantViolation {
    /// Failure trace
    pub trace: StateTrace,

    /// Log of system events
    pub log: Log,

    /// Report, which is returned by user invariant func [`crate::mc::InvariantFn`].
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

/// The liveness proprety violation:
/// goal was not achieved and there are no pending events,
/// so the system is in the terminal state.
#[derive(Clone)]
pub struct LivenessViolation {
    /// Failure trace
    pub trace: StateTrace,

    /// Log of system events
    pub log: Log,

    /// [`crate::mc::GoalFn`] report.
    pub report: String,
}

impl LivenessViolation {
    pub(crate) fn new(trace: StateTrace, log: Log, report: String) -> Self {
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

/// All states are pruned and no one state was collected.
#[derive(Clone)]
pub struct AllPruned {
    /// Last pruned state.
    pub last_trace: StateTrace,

    /// Log of the last pruned state.
    pub last_log: Log,
}

impl AllPruned {
    pub(crate) fn new(last_trace: StateTrace, last_log: Log) -> Self {
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

/// The system is in cycle from which can not out.
#[derive(Clone)]
pub struct Cycled {
    /// Cycle trace
    pub trace: StateTrace,

    /// Log of system events
    pub log: Log,

    /// Hash of the state, which allows to see equivalant states, which make the cycle.
    pub hash: HashType,
}

impl Cycled {
    pub(crate) fn new(trace: StateTrace, log: Log, hash: HashType) -> Self {
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

/// Represents search error kind.
#[derive(Clone)]
pub enum SearchErrorKind {
    /// Invariant violation
    InvariantViolation(InvariantViolation),

    /// Liveness property violation
    LivenessViolation(LivenessViolation),

    /// All states are pruned during collect.
    AllPruned(AllPruned),

    /// Some process panic
    ProcessPanic(ProcessPanic),

    /// System is in cycle.
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

/// Represents model checking error, which happens during the search.
#[derive(Clone)]
pub struct SearchError {
    /// Kind fo the error.
    pub kind: SearchErrorKind,

    /// Log of the search.
    pub log: SearchLog,
}

impl SearchError {
    pub(crate) fn new(kind: SearchErrorKind, log: &SearchLog) -> Self {
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
