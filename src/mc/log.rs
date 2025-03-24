use std::fmt::Display;

use super::event::Event;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
pub struct Log(pub Vec<Event>);

////////////////////////////////////////////////////////////////////////////////

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e);
        }
        Ok(())
    }
}
