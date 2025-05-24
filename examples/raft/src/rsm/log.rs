use std::{cell::RefCell, rc::Rc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::cmd::Command;

use super::StateHandle;

////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LogEntry {
    pub term: u64,
    pub cmd: Command,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Hash, Clone)]
pub struct Log {
    entries: Vec<LogEntry>,
}

impl Log {
    pub async fn new() -> Self {
        let entries = if let Ok(mut file) = mc::File::open("log.txt").await {
            let mut buf = [0u8; 4096];
            let bytes = file.read(&mut buf, 0).await.unwrap();
            if bytes == 0 {
                Vec::default()
            } else {
                serde_json::from_slice(&buf[..bytes]).unwrap()
            }
        } else {
            let _ = mc::File::create("log.txt").await;
            Vec::default()
        };
        Self { entries }
    }

    pub fn append_from_leader(
        &mut self,
        prev_index: usize,
        prev_term: u64,
        mut entries: Vec<LogEntry>,
    ) -> Option<mc::JoinHandle<()>> {
        if prev_index == 0 && prev_term != 0 {
            return None;
        }
        if prev_index > self.entries.len() {
            return None;
        }
        if prev_index != 0 && self.entries[prev_index - 1].term != prev_term {
            return None;
        }
        let mut equals = true;
        let len = (self.entries.len() - prev_index).min(entries.len());
        for (i, entry) in entries.iter().enumerate().take(len) {
            if *entry != self.entries[prev_index + i] {
                equals = false;
                break;
            }
        }

        // not need to change something
        let wrote_new = if !equals {
            while self.entries.len() > prev_index {
                self.entries.pop().unwrap();
            }
            self.entries.append(&mut entries);
            true
        } else {
            for entry in &entries[len..] {
                self.entries.push(entry.clone());
            }
            len < entries.len()
        };
        let handle = if wrote_new {
            let content = serde_json::to_vec(&self.entries).unwrap();
            mc::spawn(async move {
                if let Ok(mut file) = mc::File::open("log.txt").await {
                    file.write(content.as_slice(), 0).await.unwrap();
                }
            })
        } else {
            mc::spawn(async {})
        };
        Some(handle)
    }

    pub fn append_from_user(&mut self, entry: LogEntry) -> mc::JoinHandle<()> {
        mc::log("append from user");
        self.entries.push(entry);
        let content = serde_json::to_vec(&self.entries).unwrap();
        mc::spawn(async move {
            if let Ok(mut file) = mc::File::open("log.txt").await {
                file.write(content.as_slice(), 0).await.unwrap();
            }
        })
    }

    pub fn log_term(&self, index: usize) -> u64 {
        if index == 0 {
            0
        } else {
            self.entry(index).term
        }
    }

    pub fn last_log_index(&self) -> usize {
        self.entries.len()
    }

    pub fn last_log_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn entry(&self, index: usize) -> &LogEntry {
        assert!(index > 0);
        &self.entries[index - 1]
    }

    pub fn entries(&self, from: usize, to: usize) -> &[LogEntry] {
        assert!(from > 0 && to > 0);
        &self.entries[from - 1..to]
    }
}

impl From<Log> for Vec<LogEntry> {
    fn from(value: Log) -> Self {
        value.entries
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn replicate_log_for(state: StateHandle, i: usize) -> Result<bool, u64> {
    if i == state.me() {
        let Some(req) = state.make_append_request_for_follower(i) else {
            return Ok(false);
        };
        let last_index = req.prev_log_index + req.entries.len();
        state.upd_follower_info(i, last_index + 1, last_index);
        let result = state.upd_commit_index_and_apply_log();
        return Ok(result);
    }
    while let Some(req) = state.make_append_request_for_follower(i) {
        let result = req.send(i).await;
        if let Ok(result) = result {
            if !result.success && state.current_term() < result.term {
                return Err(result.term);
            } else if !result.success {
                state.dec_next_index(i);
            } else {
                let last_index = req.prev_log_index + req.entries.len();
                state.upd_follower_info(i, last_index + 1, last_index);
                let result = state.upd_commit_index_and_apply_log();
                return Ok(result);
            }
        } else {
            mc::sleep(Duration::from_millis(100)).await;
        }
    }
    Ok(false)
}

////////////////////////////////////////////////////////////////////////////////

struct Counter {
    rem: usize,
    sender: Option<mc::oneshot::Sender<Result<bool, u64>>>,
}

impl Counter {
    pub fn register(&mut self, result: Result<bool, u64>) {
        match &result {
            Ok(true) | Err(_) => {
                self.send(result);
            }
            _ => {
                self.rem -= 1;
                if self.rem == 0 {
                    self.send(Ok(false));
                }
            }
        }
    }

    pub fn send(&mut self, r: Result<bool, u64>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(r);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub async fn replicate_log(state: StateHandle) -> Result<(), u64> {
    loop {
        let (sender, receiver) = mc::oneshot::channel();
        let counter = Counter {
            rem: state.nodes(),
            sender: Some(sender),
        };
        // for myself
        let counter = Rc::new(RefCell::new(counter));
        let cancel_set = (0..state.nodes()).map(|i| {
            mc::spawn({
                let counter = counter.clone();
                let state = state.clone();
                async move {
                    let result = replicate_log_for(state, i).await;
                    counter.borrow_mut().register(result);
                }
            })
        });
        let _cancel_set = mc::CancelSet::from_iter(cancel_set);
        let result = receiver.await.unwrap();
        match result {
            Ok(true) => continue,
            Ok(false) => {
                break;
            }
            Err(term) => {
                return Err(term);
            }
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

pub async fn replicate_log_with_result(state: StateHandle) {
    let result = replicate_log(state.clone()).await;
    if let Err(term) = result {
        state.transit_to_follower(term, None, None).await;
    }
}
