use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, mpsc::UnboundedSender, oneshot};

use crate::cmd::{Command, CommandKind, Error, Response, ResponseKind};

////////////////////////////////////////////////////////////////////////////////

pub struct CommandRegistry {
    req: HashMap<usize, oneshot::Sender<Result<ResponseKind, Error>>>,
    last_id: usize,
    leader: usize,
    sender: UnboundedSender<Command>,
}

impl CommandRegistry {
    pub fn new(leader: usize, sender: UnboundedSender<Command>) -> Self {
        Self {
            req: Default::default(),
            last_id: 0,
            leader,
            sender,
        }
    }

    pub fn into_handle(self) -> CommandRegistryHandle {
        CommandRegistryHandle(Arc::new(Mutex::new(self)))
    }
}

#[derive(Clone)]
pub struct CommandRegistryHandle(Arc<Mutex<CommandRegistry>>);

impl CommandRegistryHandle {
    pub async fn register(
        &self,
        cmd: CommandKind,
    ) -> oneshot::Receiver<Result<ResponseKind, Error>> {
        let mut state = self.0.lock().await;

        let cmd = Command {
            id: state.last_id,
            leader: state.leader,
            kind: cmd,
        };

        let (sender, receiver) = oneshot::channel();
        let prev = state.req.insert(cmd.id, sender);
        assert!(prev.is_none());

        state.last_id += 1;
        let _ = state.sender.send(cmd);

        receiver
    }

    pub async fn on_response(&self, resp: Response) {
        let mut state = self.0.lock().await;
        if let Some(sender) = state.req.remove(&resp.id) {
            let _ = sender.send(resp.kind);
        }
    }
}
