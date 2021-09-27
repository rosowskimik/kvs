#![deny(missing_docs)]

//! A simple key-value store.

mod command;
mod error;
mod kvs;
mod utils;

pub use crate::kvs::KvStore;
pub use error::{KvsError, Result};

pub(crate) use command::{Command, CommandPointer};
pub(crate) use utils::{get_generation_list, logfile_path};
