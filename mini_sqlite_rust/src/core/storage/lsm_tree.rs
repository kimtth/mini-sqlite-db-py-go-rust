/// In-memory placeholder for an LSM-style commit log.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub db: String,
    pub command: String,
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}

pub struct LSMTreeStorage {
    segments: Vec<LogEntry>,
}

impl LSMTreeStorage {
    pub fn new() -> Self {
        LSMTreeStorage {
            segments: Vec::new(),
        }
    }

    /// Record a mutation event.
    pub fn log(&mut self, entry: LogEntry) {
        self.segments.push(entry);
    }

    /// Return the number of uncommitted entries.
    pub fn pending(&self) -> usize {
        self.segments.len()
    }

    /// Return a copy of the current pending entries.
    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.segments.clone()
    }

    /// Flush all pending entries and compact the log.
    pub fn commit(&mut self) -> Vec<LogEntry> {
        let flushed = self.segments.clone();
        self.segments.clear();
        self.compact();
        flushed
    }

    /// Retain only a limited window of committed history.
    fn compact(&mut self) {
        if self.segments.len() > 10 {
            self.segments = self.segments[self.segments.len() - 10..].to_vec();
        }
    }
}

impl Default for LSMTreeStorage {
    fn default() -> Self {
        Self::new()
    }
}
