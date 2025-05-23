use std::collections::HashMap;

use crate::cmd::{CommandKind, ResponseKind};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct DataBase {
    data: HashMap<String, String>,
}

impl DataBase {
    pub fn apply(&mut self, cmd: &CommandKind) -> ResponseKind {
        match cmd {
            CommandKind::Read { key } => ResponseKind::Read {
                value: self.data.get(key).cloned(),
            },
            CommandKind::Insert { key, value } => ResponseKind::Insert {
                prev: self.data.insert(key.clone(), value.clone()),
            },
            CommandKind::CAS { key, old, new } => {
                if self.data.get(key).is_some_and(|v| v == old) {
                    self.data.insert(key.clone(), new.clone());
                    ResponseKind::CAS { complete: true }
                } else {
                    ResponseKind::CAS { complete: false }
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn basic() {
        let mut db = DataBase::default();

        let r = db.apply(&CommandKind::Read { key: "k".into() });
        assert!(matches!(r, ResponseKind::Read { value: None }));

        let r = db.apply(&CommandKind::Insert {
            key: "k".into(),
            value: "v".into(),
        });
        assert!(matches!(r, ResponseKind::Insert { prev: None }));

        let r = db.apply(&CommandKind::Insert {
            key: "k".into(),
            value: "v1".into(),
        });
        assert_eq!(
            r,
            ResponseKind::Insert {
                prev: Some("v".into())
            }
        );

        let r = db.apply(&CommandKind::CAS {
            key: "k".into(),
            old: "v1".into(),
            new: "v2".into(),
        });
        assert_eq!(r, ResponseKind::CAS { complete: true });

        let r = db.apply(&CommandKind::CAS {
            key: "k".into(),
            old: "v1".into(),
            new: "v2".into(),
        });
        assert_eq!(r, ResponseKind::CAS { complete: false });

        let r = db.apply(&CommandKind::Read { key: "k".into() });
        assert_eq!(
            r,
            ResponseKind::Read {
                value: Some("v2".into()),
            },
        );
    }
}
