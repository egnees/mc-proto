use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use crate::{util::trigger::Trigger, Address};

use super::{
    error::TcpError,
    registry::TcpRegistry,
    stream::{make_connection, TcpStream},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct TcpConnectionManager {
    listeners: HashMap<Address, Trigger>,
    listeners_to: HashMap<(Address, Address), Trigger>,
}

impl TcpConnectionManager {
    fn process_streams_with_trigger(
        mut s1: TcpStream,
        mut s2: TcpStream,
        trigger: Trigger,
    ) -> Result<TcpStream, TcpError> {
        type R = Result<TcpStream, TcpError>;
        s2.mark_connected();
        let invoke_result = trigger.invoke::<R>(Ok(s2));
        if let Err(s2) = invoke_result {
            let mut s2 = s2.unwrap();
            s2.unmark_connected();
            Err(TcpError::ConnectionRefused)
        } else {
            s1.mark_connected();
            Ok(s1)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn process_listen_entry<T>(
        entry: Entry<'_, T, Trigger>,
        trigger: Trigger,
    ) -> Result<(), TcpError> {
        match entry {
            Entry::Occupied(mut e) => {
                let has_waiter = e.get().has_waiter();
                if has_waiter {
                    Err(TcpError::AlreadyListening)
                } else {
                    e.insert(trigger);
                    Ok(())
                }
            }
            Entry::Vacant(e) => {
                e.insert(trigger);
                Ok(())
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn connect(
        &mut self,
        from: &Address,
        to: &Address,
        stream_id: usize,
        registry_ref: Rc<RefCell<dyn TcpRegistry>>,
    ) -> Result<TcpStream, TcpError> {
        let (s1, s2) = make_connection(from.clone(), to.clone(), stream_id, registry_ref);
        if let Some(e) = self.listeners_to.remove(&(to.clone(), from.clone())) {
            Self::process_streams_with_trigger(s1, s2, e)
        } else if let Some(e) = self.listeners.remove(to) {
            Self::process_streams_with_trigger(s1, s2, e)
        } else {
            Err(TcpError::ConnectionRefused)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn listen_to(
        &mut self,
        on: &Address,
        to: &Address,
        on_listen: Trigger,
    ) -> Result<(), TcpError> {
        let entry = self.listeners_to.entry((on.clone(), to.clone()));
        Self::process_listen_entry(entry, on_listen)
    }

    ////////////////////////////////////////////////////////////////////////////////

    pub fn listen(&mut self, on: &Address, on_listen: Trigger) -> Result<(), TcpError> {
        let entry = self.listeners.entry(on.clone());
        Self::process_listen_entry(entry, on_listen)
    }
}
