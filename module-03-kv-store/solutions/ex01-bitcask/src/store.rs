use crate::error::{KvError, Result};
use crate::log::{open_log_file, LogEntry, LogReader, LogWriter};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct IndexEntry {
    file_id: u32,
    offset: u64,
}

pub struct KvStore {
    dir: PathBuf,
    index: HashMap<Vec<u8>, IndexEntry>,
    writer: LogWriter,
    current_file_id: u32,
    uncompacted_size: u64,
}

impl KvStore {
    const COMPACTION_THRESHOLD: u64 = 1024 * 1024; // 1MB

    pub fn open(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)?;

        let mut index = HashMap::new();
        let mut max_file_id = 0;
        let mut uncompacted_size = 0;

        // Find all log files
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "log" {
                    if let Some(stem) = path.file_stem() {
                        if let Ok(file_id) = stem.to_string_lossy().parse::<u32>() {
                            max_file_id = max_file_id.max(file_id);

                            let file = open_log_file(&path)?;
                            let mut reader = LogReader::new(file);
                            let entries = reader.read_all()?;

                            for (offset, entry) in entries {
                                match entry {
                                    LogEntry::Set { ref key, ref value } => {
                                        index.insert(
                                            key.clone(),
                                            IndexEntry { file_id, offset },
                                        );
                                        uncompacted_size += 8 + key.len() as u64 + value.len() as u64;
                                    }
                                    LogEntry::Delete { ref key } => {
                                        index.remove(key);
                                        uncompacted_size += 8 + key.len() as u64;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let current_file_id = max_file_id + 1;
        let log_path = Self::log_path(dir, current_file_id);
        let file = open_log_file(&log_path)?;
        let writer = LogWriter::new(file)?;

        Ok(KvStore {
            dir: dir.to_path_buf(),
            index,
            writer,
            current_file_id,
            uncompacted_size,
        })
    }

    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let entry = LogEntry::Set {
            key: key.to_vec(),
            value: value.to_vec(),
        };

        let (offset, size) = self.writer.append(&entry)?;
        self.index.insert(
            key.to_vec(),
            IndexEntry {
                file_id: self.current_file_id,
                offset,
            },
        );

        self.uncompacted_size += size as u64;

        if self.uncompacted_size > Self::COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some(entry) = self.index.get(key) {
            let log_path = Self::log_path(&self.dir, entry.file_id);
            let file = open_log_file(&log_path)?;
            let mut reader = LogReader::new(file);

            match reader.read_at(entry.offset)? {
                LogEntry::Set { key: _, value } => Ok(Some(value)),
                LogEntry::Delete { .. } => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        if !self.index.contains_key(key) {
            return Ok(());
        }

        let entry = LogEntry::Delete {
            key: key.to_vec(),
        };

        let (_, size) = self.writer.append(&entry)?;
        self.index.remove(key);
        self.uncompacted_size += size as u64;

        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        let compaction_file_id = self.current_file_id + 1;
        let compaction_path = Self::log_path(&self.dir, compaction_file_id);

        let file = open_log_file(&compaction_path)?;
        let mut compaction_writer = LogWriter::new(file)?;

        let mut new_index = HashMap::new();

        for (key, _) in &self.index {
            if let Some(value) = self.get(key)? {
                let entry = LogEntry::Set {
                    key: key.clone(),
                    value,
                };

                let (offset, _) = compaction_writer.append(&entry)?;
                new_index.insert(
                    key.clone(),
                    IndexEntry {
                        file_id: compaction_file_id,
                        offset,
                    },
                );
            }
        }

        compaction_writer.sync()?;

        // Remove old log files
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "log" {
                    if let Some(stem) = path.file_stem() {
                        if let Ok(file_id) = stem.to_string_lossy().parse::<u32>() {
                            if file_id <= self.current_file_id {
                                fs::remove_file(&path)?;
                            }
                        }
                    }
                }
            }
        }

        self.index = new_index;
        self.current_file_id = compaction_file_id + 1;
        self.uncompacted_size = 0;

        let new_log_path = Self::log_path(&self.dir, self.current_file_id);
        let new_file = open_log_file(&new_log_path)?;
        self.writer = LogWriter::new(new_file)?;

        Ok(())
    }

    fn log_path(dir: &Path, file_id: u32) -> PathBuf {
        dir.join(format!("{}.log", file_id))
    }
}
