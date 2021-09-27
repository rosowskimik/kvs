use std::ops::Range;

use serde::{Deserialize, Serialize};

/// Represents [`KvStore`] commands that are persisted to disk.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Command {
    Set(String, String),
    Remove(String),
}

impl Command {
    /// Get the [`&str`] description of command
    pub(crate) fn kind(&self) -> &'static str {
        match *self {
            Self::Set(_, _) => "set",
            Self::Remove(_) => "rm",
        }
    }
}

/// An in-memory representation that stores the generation
/// and in-file position of a `Command`.
#[derive(Debug)]
pub(crate) struct CommandPointer {
    gen: usize,
    start: usize,
    length: usize,
}

impl CommandPointer {
    pub(crate) fn new(gen: usize, range: Range<usize>) -> Self {
        Self {
            start: range.start,
            length: range.len(),
            gen,
        }
    }

    pub(crate) fn start(&self) -> usize {
        self.start
    }
    pub(crate) fn len(&self) -> usize {
        self.length
    }

    pub(crate) fn gen(&self) -> usize {
        self.gen
    }
}
