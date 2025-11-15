use crate::error::{QueueError, Result};
use crate::message::{LogEntry, Message, MessageStatus};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Persistent log store for messages
pub struct LogStore {
    path: PathBuf,
    writer: BufWriter<File>,
    /// Maps message_id -> file offset
    index: HashMap<String, u64>,
    /// Current write offset
    offset: u64,
}

impl LogStore {
    /// Create or open a log store at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file_exists = path.exists();

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(&path)?;

        let mut store = LogStore {
            path: path.clone(),
            writer: BufWriter::new(file),
            index: HashMap::new(),
            offset: 0,
        };

        if file_exists {
            info!("Recovering log from {:?}", path);
            store.recover()?;
        }

        Ok(store)
    }

    /// Append a new log entry
    pub fn append(&mut self, message: &Message, status: MessageStatus) -> Result<()> {
        let entry = LogEntry::new(message.clone(), status);
        self.append_entry(&entry)
    }

    /// Append a log entry to the file
    fn append_entry(&mut self, entry: &LogEntry) -> Result<()> {
        let data = bincode::serialize(entry)?;
        let len = data.len() as u32;

        // Write length prefix (4 bytes) then data
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&data)?;
        self.writer.flush()?;

        // Update index
        self.index.insert(entry.message.id.clone(), self.offset);
        self.offset += 4 + len as u64;

        debug!(
            "Appended message {} at offset {} with status {:?}",
            entry.message.id, self.offset, entry.status
        );

        Ok(())
    }

    /// Mark a message as acknowledged
    pub fn mark_acked(&mut self, msg_id: &str) -> Result<()> {
        if let Some(msg) = self.read_message(msg_id)? {
            let entry = LogEntry::new(msg, MessageStatus::Acknowledged);
            self.append_entry(&entry)?;
        }
        Ok(())
    }

    /// Mark a message as failed
    pub fn mark_failed(&mut self, msg_id: &str) -> Result<()> {
        if let Some(msg) = self.read_message(msg_id)? {
            let entry = LogEntry::new(msg, MessageStatus::Failed);
            self.append_entry(&entry)?;
        }
        Ok(())
    }

    /// Mark a message as delivered
    pub fn mark_delivered(&mut self, msg_id: &str) -> Result<()> {
        if let Some(msg) = self.read_message(msg_id)? {
            let entry = LogEntry::new(msg, MessageStatus::Delivered);
            self.append_entry(&entry)?;
        }
        Ok(())
    }

    /// Read a message by ID
    fn read_message(&self, msg_id: &str) -> Result<Option<Message>> {
        if let Some(&offset) = self.index.get(msg_id) {
            let mut reader = BufReader::new(File::open(&self.path)?);
            reader.seek(SeekFrom::Start(offset))?;

            // Read length
            let mut len_bytes = [0u8; 4];
            reader.read_exact(&mut len_bytes)?;
            let len = u32::from_le_bytes(len_bytes);

            // Read data
            let mut data = vec![0u8; len as usize];
            reader.read_exact(&mut data)?;

            let entry: LogEntry = bincode::deserialize(&data)?;
            Ok(Some(entry.message))
        } else {
            Ok(None)
        }
    }

    /// Recover the log by scanning all entries
    /// Returns all pending messages that need to be redelivered
    pub fn recover(&mut self) -> Result<Vec<Message>> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        let mut pending = HashMap::new();
        let mut offset = 0u64;

        loop {
            // Try to read length prefix
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of file
                    break;
                }
                Err(e) => return Err(QueueError::from(e)),
            }

            let len = u32::from_le_bytes(len_bytes);

            // Read entry data
            let mut data = vec![0u8; len as usize];
            reader.read_exact(&mut data)?;

            match bincode::deserialize::<LogEntry>(&data) {
                Ok(entry) => {
                    let msg_id = entry.message.id.clone();

                    // Update index
                    self.index.insert(msg_id.clone(), offset);

                    // Update message status
                    match entry.status {
                        MessageStatus::Pending | MessageStatus::Delivered => {
                            // Message needs redelivery
                            pending.insert(msg_id, entry.message);
                        }
                        MessageStatus::Acknowledged | MessageStatus::DeadLettered => {
                            // Message is done, remove from pending
                            pending.remove(&msg_id);
                        }
                        MessageStatus::Failed => {
                            // Keep in pending for retry
                            pending.insert(msg_id, entry.message);
                        }
                    }

                    offset += 4 + len as u64;
                }
                Err(e) => {
                    // Corrupted entry, skip
                    debug!("Skipping corrupted entry at offset {}: {}", offset, e);
                    offset += 4 + len as u64;
                }
            }
        }

        self.offset = offset;

        let pending_messages: Vec<Message> = pending.into_values().collect();
        info!("Recovered {} pending messages", pending_messages.len());

        Ok(pending_messages)
    }

    /// Compact the log by removing acknowledged messages
    /// This creates a new file with only pending/failed messages
    pub fn compact(&mut self) -> Result<()> {
        info!("Compacting log at {:?}", self.path);

        let temp_path = self.path.with_extension("tmp");
        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)?;

        let mut new_index = HashMap::new();
        let mut new_offset = 0u64;

        // Read all entries and write only pending/failed ones to temp file
        let mut reader = BufReader::new(File::open(&self.path)?);
        let mut seen_messages: HashMap<String, LogEntry> = HashMap::new();

        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(QueueError::from(e)),
            }

            let len = u32::from_le_bytes(len_bytes);
            let mut data = vec![0u8; len as usize];
            reader.read_exact(&mut data)?;

            if let Ok(entry) = bincode::deserialize::<LogEntry>(&data) {
                seen_messages.insert(entry.message.id.clone(), entry);
            }
        }

        // Write only pending/failed messages to new file
        for entry in seen_messages.values() {
            match entry.status {
                MessageStatus::Pending | MessageStatus::Delivered | MessageStatus::Failed => {
                    let data = bincode::serialize(entry)?;
                    let len = data.len() as u32;

                    temp_file.write_all(&len.to_le_bytes())?;
                    temp_file.write_all(&data)?;

                    new_index.insert(entry.message.id.clone(), new_offset);
                    new_offset += 4 + len as u64;
                }
                _ => {}
            }
        }

        temp_file.flush()?;
        drop(temp_file);

        // Replace old file with new file
        std::fs::rename(&temp_path, &self.path)?;

        // Reopen file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.path)?;

        self.writer = BufWriter::new(file);
        self.index = new_index;
        self.offset = new_offset;

        info!("Compaction complete. New offset: {}", new_offset);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_append_and_recover() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        {
            let mut log = LogStore::open(&path).unwrap();

            let msg1 = Message::new("test", b"message1".to_vec());
            let msg2 = Message::new("test", b"message2".to_vec());

            log.append(&msg1, MessageStatus::Pending).unwrap();
            log.append(&msg2, MessageStatus::Pending).unwrap();
            log.mark_acked(&msg1.id).unwrap();
        }

        // Reopen and recover
        {
            let mut log = LogStore::open(&path).unwrap();
            let pending = log.recover().unwrap();

            // Only msg2 should be pending
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].payload, b"message2");
        }
    }

    #[test]
    fn test_mark_acked() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let mut log = LogStore::open(&path).unwrap();

        let msg = Message::new("test", b"data".to_vec());
        log.append(&msg, MessageStatus::Pending).unwrap();
        log.mark_acked(&msg.id).unwrap();

        drop(log);

        let mut log = LogStore::open(&path).unwrap();
        let pending = log.recover().unwrap();

        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_compact() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let mut log = LogStore::open(&path).unwrap();

        // Add 10 messages, ack 5
        for i in 0..10 {
            let msg = Message::new("test", format!("msg{}", i).into_bytes());
            log.append(&msg, MessageStatus::Pending).unwrap();
            if i < 5 {
                log.mark_acked(&msg.id).unwrap();
            }
        }

        let size_before = std::fs::metadata(&path).unwrap().len();

        log.compact().unwrap();

        let size_after = std::fs::metadata(&path).unwrap().len();

        // File should be smaller after compaction
        assert!(size_after < size_before);

        // Should have 5 pending messages
        drop(log);
        let mut log = LogStore::open(&path).unwrap();
        let pending = log.recover().unwrap();
        assert_eq!(pending.len(), 5);
    }
}
