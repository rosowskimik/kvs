use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem;
use std::path::{Path, PathBuf};

use crate::{
    command::Command,
    get_generation_list, logfile_path,
    utils::{get_logfile, replay},
    CommandPointer, KvsError, Result,
};

const SIZE_THRESHOLD: usize = 1024 * 1024;

/// The [`KvStore`] stores string key-value pairs.
///
/// Key-value pairs are persisted to disk in log files. Log files
/// are named after increasing generation number with `log` extension.
/// An in-memory [`HashMap`] is used for quick key-value on-disk location lookup.
///
/// # Examples
///
/// ```rust no_run
/// # use kvs::{Result, KvStore};
/// # fn main() -> Result<()> {
/// use std::env::current_dir;
/// let mut store = KvStore::open(current_dir()?)?;
///
/// store.set("key", "value")?;
/// assert_eq!(store.get("key")?, Some("value".to_string()));
///
/// store.remove("key")?;
/// assert_eq!(store.get("key")?, None);
///
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct KvStore {
    path: PathBuf,
    index: HashMap<String, CommandPointer>,
    readers: HashMap<usize, BufReader<File>>,
    writer: BufWriter<File>,
    curr_gen: usize,
    stale_bytes: usize,
}

impl KvStore {
    /// Opens a [`KvStore`] within provided `path`.
    ///
    /// This will create a new store directory if the given one doesn't exist.
    ///
    /// # Errors
    ///
    /// This function propagates I/O and deserialization errors that could arise during log replay.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        fs::create_dir_all(&path)?;

        let prev_gens = get_generation_list(&path)?;

        let curr_gen = if let Some(last_gen) = prev_gens.last().copied() {
            let last_logfile_path = logfile_path(&path, last_gen);
            if fs::metadata(last_logfile_path)?.len() <= SIZE_THRESHOLD as u64 {
                last_gen
            } else {
                last_gen.wrapping_add(1)
            }
        } else {
            1
        };

        let mut stale_bytes = 0;
        let mut index = HashMap::new();
        let mut readers = HashMap::with_capacity(prev_gens.len() + 1);

        for gen in prev_gens {
            let mut reader = BufReader::new(File::open(logfile_path(&path, gen))?);

            stale_bytes += replay(&mut reader, &mut index, gen)?;

            readers.insert(gen, reader);
        }

        let current_logfile = get_logfile(&path, curr_gen)?;
        readers.insert(curr_gen, BufReader::new(current_logfile.try_clone()?));

        let writer = BufWriter::new(current_logfile);

        Ok(Self {
            path: PathBuf::from(path.as_ref()),
            curr_gen,
            readers,
            writer,
            index,
            stale_bytes,
        })
    }

    /// Sets the given `key` to provided `value`.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// This function propagates serialization and I/O errors that could arise while
    /// writing to the log.
    pub fn set<K, V>(&mut self, key: K, value: V) -> Result<()>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key = key.into();
        let value = value.into();
        let start = self.writer.stream_position()? as usize;

        let command = Command::Set(key, value);
        serde_json::to_writer(&mut self.writer, &command)?;

        let end = self.writer.stream_position()? as usize;

        let cmd_ptr = CommandPointer::new(self.curr_gen, start..end);

        if let Command::Set(key, _) = command {
            if let Some(old_cmd_ptr) = self.index.insert(key, cmd_ptr) {
                self.stale_bytes += old_cmd_ptr.len();
            }
        }

        if self.stale_bytes > SIZE_THRESHOLD {
            self.clean_stale_data()?;
        }

        Ok(())
    }

    /// Fetches the stored `value` of a given `key`.
    ///
    /// Returns [`None`] if the key does not exist.
    ///
    /// # Errors
    ///
    /// This function propagates deserialization and I/O errors that could arise while
    /// reading the log.
    pub fn get<K: Into<String>>(&mut self, key: K) -> Result<Option<String>> {
        if let Some(cmd_ptr) = self.index.get(&key.into()) {
            let gen = cmd_ptr.gen();
            let start = cmd_ptr.start();
            let length = cmd_ptr.len();

            let logfile = self
                .readers
                .get_mut(&gen)
                .ok_or_else(|| KvsError::MissingLogfile(gen))?;

            logfile.seek(SeekFrom::Start(start as u64))?;
            let reader = logfile.take(length as u64);

            let command: Command = serde_json::from_reader(reader)?;

            if let Command::Set(_, value) = command {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommand {
                    expected: "set",
                    got: command.kind(),
                })
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a given key returning `true` if the key was saved, `false` otherwise.
    ///
    /// # Errors
    ///
    /// This function propagates serialization and I/O errors that could arise while
    /// writing to the log.
    pub fn remove<K: Into<String>>(&mut self, key: K) -> Result<bool> {
        let key = key.into();

        let command = Command::Remove(key);

        serde_json::to_writer(&mut self.writer, &command)?;

        if let Command::Remove(key) = command {
            if let Some(old_cmd_ptr) = self.index.remove(&key) {
                self.stale_bytes += old_cmd_ptr.len();
                if self.stale_bytes > SIZE_THRESHOLD {
                    self.clean_stale_data()?;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            unreachable!()
        }
    }

    /// Removes all stale data from the disk.
    ///
    /// # Errors
    /// This function propagates any I/O error that could arise while
    /// writing to the disk. The process itself guarantees that no data
    /// will be lost in case of a crash during cleanup.
    pub fn clean_stale_data(&mut self) -> Result<usize> {
        self.flush()?;

        let stale = self.stale_bytes;

        let clean_gen = self.curr_gen.wrapping_add(1);
        let clean_file = get_logfile(&self.path, clean_gen)?;
        let mut clean_writer = BufWriter::new(clean_file.try_clone()?);

        let mut clean_start = 0;

        for cmd_ptr in self.index.values_mut() {
            let logfile = self
                .readers
                .get_mut(&cmd_ptr.gen())
                .ok_or_else(|| KvsError::MissingLogfile(cmd_ptr.gen()))?;

            logfile.seek(SeekFrom::Start(cmd_ptr.start() as u64))?;

            let mut reader = logfile.take(cmd_ptr.len() as u64);

            let length = io::copy(&mut reader, &mut clean_writer)? as usize;
            *cmd_ptr = CommandPointer::new(clean_gen, clean_start..clean_start + length);

            clean_start += length;
        }
        clean_writer.flush()?;

        let mut new_readers = HashMap::new();
        new_readers.insert(clean_gen, BufReader::new(clean_file));

        if clean_writer.get_ref().metadata()?.len() > SIZE_THRESHOLD as u64 {
            let new_gen = self.curr_gen.wrapping_add(2);
            let new_logfile = get_logfile(&self.path, self.curr_gen)?;
            let new_writer = BufWriter::new(new_logfile.try_clone()?);

            new_readers.insert(new_gen, BufReader::new(new_logfile));

            self.curr_gen = new_gen;
            self.writer = new_writer;
        } else {
            self.curr_gen = clean_gen;
            self.writer = clean_writer;
        }

        let stale_readers = mem::replace(&mut self.readers, new_readers);

        stale_readers
            .into_keys()
            .try_for_each(|stale_gen| -> Result<()> {
                let path = logfile_path(&self.path, stale_gen);
                fs::remove_file(path)?;
                Ok(())
            })?;

        self.stale_bytes = 0;

        Ok(stale)
    }

    /// Flushes any pending write operation to disk.
    ///
    /// # Errors
    ///
    /// This function propagates any I/O error that could arise while
    /// flushing the buffer to the disk.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
