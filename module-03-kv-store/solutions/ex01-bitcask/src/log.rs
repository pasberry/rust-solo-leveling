use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEntry {
    Set { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

pub struct LogWriter {
    writer: BufWriter<File>,
    offset: u64,
}

impl LogWriter {
    pub fn new(file: File) -> Result<Self> {
        let offset = file.metadata()?.len();
        Ok(LogWriter {
            writer: BufWriter::new(file),
            offset,
        })
    }

    pub fn append(&mut self, entry: &LogEntry) -> Result<(u64, u32)> {
        let start_offset = self.offset;
        let data = bincode::serialize(entry)?;

        let mut hasher = Hasher::new();
        hasher.update(&data);
        let crc = hasher.finalize();

        self.writer.write_u32::<LittleEndian>(crc)?;
        self.writer.write_u32::<LittleEndian>(data.len() as u32)?;
        self.writer.write_all(&data)?;
        self.writer.flush()?;

        let entry_size = 8 + data.len() as u64;
        self.offset += entry_size;

        Ok((start_offset, entry_size as u32))
    }

    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        Ok(())
    }
}

pub struct LogReader {
    reader: BufReader<File>,
}

impl LogReader {
    pub fn new(file: File) -> Self {
        LogReader {
            reader: BufReader::new(file),
        }
    }

    pub fn read_at(&mut self, offset: u64) -> Result<LogEntry> {
        self.reader.seek(SeekFrom::Start(offset))?;

        let crc = self.reader.read_u32::<LittleEndian>()?;
        let len = self.reader.read_u32::<LittleEndian>()?;

        let mut data = vec![0u8; len as usize];
        self.reader.read_exact(&mut data)?;

        let mut hasher = Hasher::new();
        hasher.update(&data);
        if hasher.finalize() != crc {
            return Err(crate::error::KvError::Corruption);
        }

        Ok(bincode::deserialize(&data)?)
    }

    pub fn read_all(&mut self) -> Result<Vec<(u64, LogEntry)>> {
        let mut entries = Vec::new();
        self.reader.seek(SeekFrom::Start(0))?;

        loop {
            let offset = self.reader.stream_position()?;

            let crc = match self.reader.read_u32::<LittleEndian>() {
                Ok(crc) => crc,
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };

            let len = self.reader.read_u32::<LittleEndian>()?;
            let mut data = vec![0u8; len as usize];
            self.reader.read_exact(&mut data)?;

            let mut hasher = Hasher::new();
            hasher.update(&data);
            if hasher.finalize() != crc {
                return Err(crate::error::KvError::Corruption);
            }

            let entry = bincode::deserialize(&data)?;
            entries.push((offset, entry));
        }

        Ok(entries)
    }
}

pub fn open_log_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .map_err(Into::into)
}
