use std::{
    fmt::Display,
    ops::{Index, RangeFrom},
};

use crate::proto::raftpb::Entry;

/// State of a raft peer.
#[derive(Default, Clone, Debug)]
pub struct State {
    pub term: u64,
    pub is_leader: bool,
}

impl State {
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.term
    }
    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}

pub fn entries_to_str(entries: &Vec<Entry>) -> String {
    let v: Vec<_> = entries.iter().map(|e| e.to_string()).collect();
    format!("len={}, {}", entries.len(), v.join(", "))
}

#[derive(Message, Clone)]
pub struct Log {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<Entry>,
    #[prost(uint64, tag = "2")]
    last_included_index: u64,
    #[prost(uint64, tag = "3")]
    last_included_term: u64,
}

impl Log {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            last_included_index: 0,
            last_included_term: 0,
        }
    }

    pub fn new_from_entries(
        entries: Vec<Entry>,
        last_included_index: u64,
        last_included_term: u64,
    ) -> Self {
        Self {
            entries,
            last_included_index,
            last_included_term,
        }
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn last_included_index(&self) -> u64 {
        self.last_included_index
    }

    pub fn last_included_term(&self) -> u64 {
        self.last_included_term
    }

    pub fn physical_len(&self) -> usize {
        self.entries.len()
    }

    // returns the logical length of the log
    pub fn len(&self) -> usize {
        let included_len = self.last_included_index as usize + 1;
        included_len + self.entries.len()
    }

    // returns if the log is empty, logically
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // return term of the last log entry
    pub fn last_term(&self) -> u64 {
        self.term_at(self.len() - 1)
    }

    // return term of entry at `idx`
    pub fn term_at(&self, idx: usize) -> u64 {
        if idx == self.last_included_index as usize {
            self.last_included_term
        } else {
            self[idx].term
        }
    }

    pub fn push(&mut self, e: Entry) {
        self.entries.push(e)
    }

    pub fn append(&mut self, xs: &mut Vec<Entry>) {
        self.entries.append(xs)
    }

    // truncate the log starting at `idx`, including `idx`
    // idx: logical index
    pub fn truncate(&mut self, idx: usize) {
        self.entries
            .truncate(idx - self.last_included_index as usize - 1)
    }

    // discard entries up to `index`, inclusive
    // `index` is logical index
    pub fn discard(&mut self, index: usize) {
        let physical_index = index - self.last_included_index as usize - 1;

        self.last_included_term = self.term_at(index);
        self.last_included_index = index as u64;

        self.entries.drain(..=physical_index);
    }
}

impl Index<usize> for Log {
    type Output = Entry;

    // idx: logical index
    fn index(&self, idx: usize) -> &Self::Output {
        if idx <= self.last_included_index as usize {
            panic!(
                "idx={}, last_included_index={}",
                idx, self.last_included_index
            )
        }

        &self.entries[idx - self.last_included_index as usize - 1]
    }
}

impl Index<RangeFrom<usize>> for Log {
    type Output = [Entry];

    fn index(&self, range_from: RangeFrom<usize>) -> &Self::Output {
        let start = range_from.start;
        if start <= self.last_included_index as usize {
            panic!(
                "start={}, last_included_index={}",
                start, self.last_included_index
            )
        }
        &self.entries[(start - self.last_included_index as usize - 1)..]
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "last_included_index={}, last_included_term={}, {}",
            self.last_included_index,
            self.last_included_term,
            entries_to_str(&self.entries)
        )
    }
}

#[derive(Message)]
pub struct PersistentState {
    #[prost(uint64, tag = "1")]
    pub term: u64,
    #[prost(uint64, optional, tag = "2")]
    pub voted_for: Option<u64>,
    // I cannot find a way to add `Log` as a field directly.
    // So I put fields separately here.
    #[prost(message, repeated, tag = "3")]
    pub entries: Vec<Entry>,
    #[prost(uint64, tag = "4")]
    pub last_included_index: u64,
    #[prost(uint64, tag = "5")]
    pub last_included_term: u64,
}
