use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::addr::make_addr;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CommandKind {
    Read {
        key: String,
    },
    Insert {
        key: String,
        value: String,
    },
    CAS {
        key: String,
        old: String,
        new: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Command {
    pub id: usize,
    pub leader: usize,
    pub kind: CommandKind,
}

impl Command {
    pub fn response(&self, kind: ResponseKind) -> Response {
        Response {
            id: self.id,
            kind: Ok(kind),
        }
    }

    pub fn response_failed(&self, leader: Option<usize>) -> Response {
        Response {
            id: self.id,
            kind: Err(Error::NotLeader {
                redirected_to: leader,
            }),
        }
    }

    pub fn read(id: usize, leader: usize, key: impl Into<String>) -> Self {
        Self {
            id,
            leader,
            kind: CommandKind::Read { key: key.into() },
        }
    }

    pub fn insert(
        id: usize,
        leader: usize,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            id,
            leader,
            kind: CommandKind::Insert {
                key: key.into(),
                value: value.into(),
            },
        }
    }

    pub fn cas(
        id: usize,
        leader: usize,
        key: impl Into<String>,
        old: impl Into<String>,
        new: impl Into<String>,
    ) -> Self {
        Self {
            id,
            leader,
            kind: CommandKind::CAS {
                key: key.into(),
                old: old.into(),
                new: new.into(),
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ResponseKind {
    Read { value: Option<String> },
    Insert { prev: Option<String> },
    CAS { complete: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: usize,
    pub kind: Result<ResponseKind, Error>,
}

impl Response {
    pub fn new_error(id: usize, error: Error) -> Self {
        Self {
            id,
            kind: Err(error),
        }
    }
}

impl From<Response> for String {
    fn from(value: Response) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    #[error("not leader; redirected_to: {:?}", redirected_to.map(make_addr))]
    NotLeader { redirected_to: Option<usize> },
}
