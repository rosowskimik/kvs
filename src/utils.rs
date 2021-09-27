use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use serde_json::Deserializer;

use crate::{Command, CommandPointer, Result};

/// Fetches all previous generations at a given path in sorted order.
pub(crate) fn get_generation_list<P: AsRef<Path>>(path: P) -> Result<Vec<usize>> {
    let mut generations: Vec<usize> = fs::read_dir(path)?
        .flat_map(|entry| -> Result<_> { Ok(entry?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("log")))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<usize>)
        })
        .filter_map(|gens: std::result::Result<usize, ParseIntError>| gens.ok())
        .collect();

    generations.sort_unstable();
    Ok(generations)
}

pub(crate) fn logfile_path<P: AsRef<Path>>(path: P, gen: usize) -> PathBuf {
    path.as_ref().join(format!("{}.log", gen))
}

/// Creates new logfile at given path with given generation number.
pub(crate) fn new_logfile<P: AsRef<Path>>(path: P, gen: usize) -> Result<File> {
    let new_path = logfile_path(path, gen);

    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(new_path)?)
}

/// Restores the in-memory index by replaying all `Command`s stored in a logfile.
///
/// This function returns the amount of stale bytes that can be recovered.
pub(crate) fn replay<R: Read + Seek>(
    mut logfile: R,
    index: &mut HashMap<String, CommandPointer>,
    gen: usize,
) -> Result<usize> {
    let (mut start, mut stale) = (0, 0);

    logfile.rewind()?;
    let mut stream = Deserializer::from_reader(logfile).into_iter::<Command>();

    while let Some(command) = stream.next() {
        let command = command?;
        let end = stream.byte_offset();

        match command {
            Command::Set(key, _) => {
                let cmd_ptr = CommandPointer::new(gen, start..end);

                if let Some(old_cmd_ptr) = index.insert(key, cmd_ptr) {
                    stale += old_cmd_ptr.len();
                }
            }
            Command::Remove(key) => {
                if let Some(old_cmd) = index.remove(&key) {
                    stale += old_cmd.len();
                }
                stale += end - start;
            }
        }

        start = end;
    }

    Ok(stale)
}
