use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub expression: String,
    pub result: String,
    pub err: Option<String>,
    pub at: i64,
}

#[derive(Clone, Debug)]
pub struct History {
    entries: Vec<Entry>,
    limit: usize,
}

impl History {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::with_capacity(limit),
            limit,
        }
    }

    pub fn add(&mut self, expr: &str, result: &str, err: Option<String>) {
        let entry = Entry {
            expression: expr.to_string(),
            result: result.to_string(),
            err,
            at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };

        self.entries.push(entry);
        if self.entries.len() > self.limit {
            let start = self.entries.len() - self.limit;
            self.entries.drain(0..start);
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn load_entries(&mut self, entries: Vec<Entry>) {
        let mut entries = entries;
        if entries.len() > self.limit {
            entries = entries.split_off(entries.len() - self.limit);
        }
        self.entries = entries;
    }

    pub fn to_vec(&self) -> Vec<Entry> {
        self.entries.clone()
    }
}
