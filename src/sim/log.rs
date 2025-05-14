use std::{fmt::Display, time::Duration};

use colored::Colorize;

use crate::{event::time::Time, fs::event::FsEventOutcome, tcp::packet::TcpPacket};

use super::proc::Address;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct OpenFileRequested {
    pub time: Time,
    pub proc: Address,
    pub file: String,
    pub outcome: FsEventOutcome,
}

impl Display for OpenFileRequested {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outcome.is_ok() {
            write!(
                f,
                "{} {:>12}   O  {:<12}",
                self.time,
                self.proc.to_string(),
                self.file.to_string()
            )
        } else {
            write!(
                f,
                "{}",
                format!(
                    "{} {:>12}   O  {:<12} <--- failed",
                    self.time,
                    self.proc.to_string(),
                    self.file.to_string()
                )
                .red(),
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct CreateFileRequested {
    pub time: Time,
    pub proc: Address,
    pub file: String,
    pub outcome: FsEventOutcome,
}

impl Display for CreateFileRequested {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outcome.is_ok() {
            write!(
                f,
                "{} {:>12}   C  {:<12}",
                self.time,
                self.proc.to_string(),
                self.file.to_string()
            )
        } else {
            write!(
                f,
                "{}",
                format!(
                    "{} {:>12}   C  {:<12} <--- failed",
                    self.time,
                    self.proc.to_string(),
                    self.file.to_string()
                )
                .red(),
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct DeleteFileRequested {
    pub time: Time,
    pub proc: Address,
    pub file: String,
    pub outcome: FsEventOutcome,
}

impl Display for DeleteFileRequested {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outcome.is_ok() {
            write!(
                f,
                "{} {:>12}   D  {:<12}",
                self.time,
                self.proc.to_string(),
                self.file.to_string()
            )
        } else {
            write!(
                f,
                "{}",
                format!(
                    "{} {:>12}   D  {:<12} <--- failed",
                    self.time,
                    self.proc.to_string(),
                    self.file.to_string()
                )
                .red(),
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ReadFileInitiated {
    pub time: Time,
    pub proc: Address,
    pub file: String,
}

impl Display for ReadFileInitiated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} R 🚀 {:<12}",
            self.time,
            self.proc.to_string(),
            self.file.to_string()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ReadFileCompleted {
    pub time: Time,
    pub proc: Address,
    pub file: String,
    pub outcome: FsEventOutcome,
}

impl Display for ReadFileCompleted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outcome.is_ok() {
            write!(
                f,
                "{} {:>12} R 🚩 {:<12}",
                self.time,
                self.proc.to_string(),
                self.file.to_string()
            )
        } else {
            write!(
                f,
                "{}",
                format!(
                    "{} {:>12} R 🚩 {:<12} <--- failed",
                    self.time,
                    self.proc.to_string(),
                    self.file.to_string()
                )
                .red(),
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct WriteFileInitiated {
    pub time: Time,
    pub proc: Address,
    pub file: String,
}

impl Display for WriteFileInitiated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} W 🚀 {:<12}",
            self.time,
            self.proc.to_string(),
            self.file.to_string()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct WriteFileCompleted {
    pub time: Time,
    pub proc: Address,
    pub file: String,
    pub outcome: FsEventOutcome,
}

impl Display for WriteFileCompleted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.outcome.is_ok() {
            write!(
                f,
                "{} {:>12} W 🚩 {:<12}",
                self.time,
                self.proc.to_string(),
                self.file.to_string()
            )
        } else {
            write!(
                f,
                "{}",
                format!(
                    "{} {:>12} W 🚩 {:<12} <--- failed",
                    self.time,
                    self.proc.to_string(),
                    self.file.to_string()
                )
                .red(),
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TcpMessageSent {
    pub from: Address,
    pub to: Address,
    pub packet: TcpPacket,
    pub time: Time,
}

impl Display for TcpMessageSent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} ---> {:<12} {:?}",
            self.time,
            self.from.to_string(),
            self.to.to_string(),
            self.packet.to_string()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TcpMessageReceived {
    pub from: Address,
    pub to: Address,
    pub packet: TcpPacket,
    pub time: Time,
}

impl Display for TcpMessageReceived {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} <--- {:<12} {:?}",
            self.time,
            self.to.to_string(),
            self.from.to_string(),
            self.packet.to_string()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TcpMessageDropped {
    pub from: Address,
    pub to: Address,
    pub packet: TcpPacket,
    pub time: Time,
}

impl Display for TcpMessageDropped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12} ---x {:<12} {:?} <-- message dropped",
                self.time,
                self.from.to_string(),
                self.to.to_string(),
                self.packet.to_string()
            )
            .red()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct RpcMessageSent {
    pub from: Address,
    pub to: Address,
    pub content: Vec<u8>,
    pub time: Time,
}

impl Display for RpcMessageSent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} ---> {:<12} {:?}",
            self.time,
            self.from.to_string(),
            self.to.to_string(),
            String::from_utf8(self.content.clone()).unwrap()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct RpcMessageReceived {
    pub from: Address,
    pub to: Address,
    pub content: Vec<u8>,
    pub time: Time,
}

impl Display for RpcMessageReceived {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} <--- {:<12} {:?}",
            self.time,
            self.to.to_string(),
            self.from.to_string(),
            String::from_utf8(self.content.clone()).unwrap()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct RpcMessageDropped {
    pub from: Address,
    pub to: Address,
    pub content: Vec<u8>,
    pub time: Time,
}

impl Display for RpcMessageDropped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} ---x {:<12} {:?} <-- message dropped",
            self.time,
            self.from.to_string(),
            self.to.to_string(),
            String::from_utf8(self.content.clone()).unwrap()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageSent {
    pub from: Address,
    pub to: Address,
    pub content: String,
    pub time: Time,
}

impl Display for UdpMessageSent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} ---> {:<12} {:?}",
            self.time,
            self.from.to_string(),
            self.to.to_string(),
            self.content
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageReceived {
    pub from: Address,
    pub to: Address,
    pub content: String,
    pub time: Time,
}

impl Display for UdpMessageReceived {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:>12} <--- {:<12} {:?}",
            self.time,
            self.to.to_string(),
            self.from.to_string(),
            self.content
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UdpMessageDropped {
    pub from: Address,
    pub to: Address,
    pub content: String,
    pub time: Time,
}

impl Display for UdpMessageDropped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12} ---x {:<12} {:?} <-- message dropped",
                self.time,
                self.from.to_string(),
                self.to.to_string(),
                self.content
            )
            .red()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct FutureFellAsleep {
    pub tag: usize,
    pub proc: Address,
    pub duration: Duration,
    pub time: Time,
}

impl Display for FutureFellAsleep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12}  😴  {:<12} {:.3}",
                self.time,
                self.proc.to_string(),
                self.tag.to_string(),
                self.duration.as_secs_f64()
            )
            .bright_blue()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct FutureWokeUp {
    pub tag: usize,
    pub proc: Address,
    pub time: Time,
}

impl Display for FutureWokeUp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12}  ⏰  {:<12}",
                self.time,
                self.proc.to_string(),
                self.tag.to_string()
            )
            .bright_blue()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TimerSet {
    pub id: usize,
    pub proc: Address,
    pub time: Time,
    pub duration: Duration,
}

impl Display for TimerSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12}  ⏳  {:<12} {:.3}",
                self.time,
                self.proc.to_string(),
                self.id.to_string(),
                self.duration.as_secs_f64()
            )
            .bright_blue()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TimerCanceled {
    pub id: usize,
    pub proc: Address,
    pub time: Time,
}

impl Display for TimerCanceled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12}  ❌  {:<12}",
                self.time,
                self.proc.to_string(),
                self.id.to_string(),
            )
            .bright_blue()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TimerFired {
    pub id: usize,
    pub proc: Address,
    pub time: Time,
}

impl Display for TimerFired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12}  🔥  {:<12}",
                self.time,
                self.proc.to_string(),
                self.id.to_string(),
            )
            .bright_blue()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessSentLocalMessage {
    pub process: Address,
    pub content: String,
    pub time: Time,
}

impl Display for ProcessSentLocalMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12} >>>> {:<12} {:?}",
                self.time,
                self.process.to_string(),
                "local",
                self.content
            )
            .green()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessReceivedLocalMessage {
    pub process: Address,
    pub content: String,
    pub time: Time,
}

impl Display for ProcessReceivedLocalMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12} <<<< {:<12} {:?}",
                self.time,
                self.process.to_string(),
                "local",
                self.content
            )
            .green()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process: Address,
    pub time: Time,
    pub content: String,
}

impl Display for ProcessInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "{} {:>12} ==== {:<12?}",
                self.time,
                self.process.to_string(),
                self.content
            )
            .purple()
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct NodeCrashed {
    pub node: String,
    pub time: Time,
}

impl Display for NodeCrashed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:>12}  💥", self.time, self.node)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum LogEntry {
    TcpMessageSent(TcpMessageSent),
    TcpMessageReceived(TcpMessageReceived),
    TcpMessageDropped(TcpMessageDropped),
    UdpMessageSent(UdpMessageSent),
    UdpMessageReceived(UdpMessageReceived),
    UdpMessageDropped(UdpMessageDropped),
    ProcessSentLocalMessage(ProcessSentLocalMessage),
    ProcessReceivedLocalMessage(ProcessReceivedLocalMessage),
    FutureFellAsleep(FutureFellAsleep),
    FutureWokeUp(FutureWokeUp),
    ProcessInfo(ProcessInfo),
    NodeCrashed(NodeCrashed),
    CreateFileRequested(CreateFileRequested),
    DeleteFileRequested(DeleteFileRequested),
    ReadFileInitiated(ReadFileInitiated),
    ReadFileCompleted(ReadFileCompleted),
    WriteFileInitiated(WriteFileInitiated),
    WriteFileCompleted(WriteFileCompleted),
    OpenFileRequested(OpenFileRequested),
    RpcMessageSent(RpcMessageSent),
    RpcMessageReceived(RpcMessageReceived),
    RpcMessageDropped(RpcMessageDropped),
    TimerFired(TimerFired),
    TimerSet(TimerSet),
    TimerCanceled(TimerCanceled),
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogEntry::TcpMessageSent(e) => write!(f, "{}", e),
            LogEntry::TcpMessageReceived(e) => write!(f, "{}", e),
            LogEntry::TcpMessageDropped(e) => write!(f, "{}", e),
            LogEntry::UdpMessageSent(e) => write!(f, "{}", e),
            LogEntry::UdpMessageReceived(e) => write!(f, "{}", e),
            LogEntry::UdpMessageDropped(e) => write!(f, "{}", e),
            LogEntry::ProcessSentLocalMessage(e) => write!(f, "{}", e),
            LogEntry::ProcessReceivedLocalMessage(e) => write!(f, "{}", e),
            LogEntry::FutureFellAsleep(e) => write!(f, "{}", e),
            LogEntry::FutureWokeUp(e) => write!(f, "{}", e),
            LogEntry::ProcessInfo(e) => write!(f, "{}", e),
            LogEntry::NodeCrashed(e) => write!(f, "{}", e),
            LogEntry::CreateFileRequested(e) => write!(f, "{}", e),
            LogEntry::DeleteFileRequested(e) => write!(f, "{}", e),
            LogEntry::ReadFileInitiated(e) => write!(f, "{}", e),
            LogEntry::ReadFileCompleted(e) => write!(f, "{}", e),
            LogEntry::WriteFileInitiated(e) => write!(f, "{}", e),
            LogEntry::WriteFileCompleted(e) => write!(f, "{}", e),
            LogEntry::OpenFileRequested(e) => write!(f, "{}", e),
            LogEntry::RpcMessageSent(e) => write!(f, "{}", e),
            LogEntry::RpcMessageReceived(e) => write!(f, "{}", e),
            LogEntry::RpcMessageDropped(e) => write!(f, "{}", e),
            LogEntry::TimerFired(e) => write!(f, "{}", e),
            LogEntry::TimerSet(e) => write!(f, "{}", e),
            LogEntry::TimerCanceled(e) => write!(f, "{}", e),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct Log {
    data: Vec<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, log_entry: LogEntry) {
        self.data.push(log_entry);
    }

    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.data.iter()
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in self.data.iter() {
            writeln!(f, "{}", e)?;
        }
        write!(f, "=======================")
    }
}
