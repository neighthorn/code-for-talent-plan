use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter, Write, Seek, SeekFrom, Read};
use std::path::PathBuf;
use std::fs::{File, OpenOptions, remove_file, rename};

use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use crate::{KvStoreError, Result};

const COMPACT_THRESHOLD: u64 = 1024 * 1024;

/// 'Command' is a enum that represents various commands
#[derive(Serialize, Deserialize)]
pub enum Command {
    Set{key: String, value: String},
    Rm{key: String},
}

/// 'KvStore' is a Hashmap that can store key-value pairs
pub struct KvStore {
    // in-memory index, stores the log pointer(file offset and command length) of the command
    index: BTreeMap<String, (u64, u64)>,
    // writer buffer for log
    writer: BufWriter<File>,
    // the path of the file
    path: PathBuf,
    // the dir of the file
    dir: PathBuf,
    // the write offset of the log file
    offset: u64,
}

impl KvStore {
    /// insert a key-value pair in KvStore
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        // construct set command
        let cmd = Command::Set{ key, value };
        // serialize the command to a string
        let write_str = serde_json::to_string_pretty(&cmd)?;
        // write command into the disk
        let len = self.writer.write(write_str.as_bytes())?;
        self.writer.flush()?;
        // take the command offset in the file as value
        let cmd_offset = self.offset;
        // update the end offset of the file
        self.offset += len as u64;
        // update the in-memory index
        if let Command::Set { key, .. } = cmd {
            self.index.insert(key, (cmd_offset, len as u64));
        }
        if self.offset > COMPACT_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }
    /// get the value for the given key
    pub fn get(&self, key: String) -> Result<Option<String>> {
        // get the offset of the latest command corresponds to key
        let res = self.index.get(&key);
        match res {
            Some((pos, len)) => {
                // make the pointer of the file to the command offset
                let mut file = File::open(self.path.clone())?;
                file.seek(SeekFrom::Start(*pos))?;
                let reader = BufReader::new(file).take(*len);
                // read command from the log file
                if let Command::Set { value, .. } = serde_json::from_reader(reader)? {
                    // return the value
                    return Ok(Some(value));
                } else {
                    return Err(KvStoreError::GetNonExistValue);
                }
            },
            None => {Ok(None)},
        }
    }
    /// reomve the key-value pair with given key
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            // construct remove command
            let cmd = Command::Rm { key };
            // serialize the command to a string
            let write_str = serde_json::to_string_pretty(&cmd)?;
            // write command into the disk
            let len = self.writer.write(write_str.as_bytes())?;
            self.writer.flush()?;
            // update the end offset of the file
            self.offset += len as u64;
            // update the in-memory index
            if let Command::Rm { key } = cmd {
                self.index.remove(&key);
            }
            if self.offset > COMPACT_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(KvStoreError::RemoveNonExistKey)
        }
        
    }

    /// open the KvStore at a given path
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // let file = OpenOptions::new()
        //                     .read(true).write(true).append(true)
        //                             .open(path.into())?;
        // let mut index = BTreeMap::new();
        // let pos = rebuild_index(file, &mut index)?;
        // Ok(KvStore { index, log_file: file, path: path.into(), offset: pos })
        let mut path = path.into();
        let dir = path.clone();
        path.push("kvstore.log");
        let mut file = if path.clone().as_path().exists() {
            OpenOptions::new().read(true).write(true).open(path.clone())?
        } else {
            OpenOptions::new().read(true).write(true).create(true).open(path.clone())?
        };
        // let mut file = OpenOptions::new()
        //                 .read(true)
        //                 .write(true)
        //                 .create(true)
        //                 .truncate(true)
        //                 .open(path.clone())?;
        let mut index= BTreeMap::new();
        let offset = rebuild_index(&file, &mut index)?;
        file.seek(SeekFrom::End(0))?;
        let writer = BufWriter::new(file);
        Ok(KvStore { index, writer, path, dir, offset})
    }

    /// used to compact the kvstore and the log, remove the redundant key-value command
    pub fn compact(&mut self) -> Result<()> {
        let mut new_file_path = self.dir.clone();
        new_file_path.push("kvstore.bk");
        // kvstore.bk
        let new_file = OpenOptions::new()
                    .read(true).write(true).create(true).open(new_file_path.clone())?;
        // kvstore.log
        let old_file = File::open(self.path.clone())?;
        let mut stream = Deserializer::from_reader(old_file).into_iter::<Command>();
        let mut new_file_writer = BufWriter::new(new_file);

        // read every command in log file
        let mut read_offset: u64 = 0;
        while let Some(cmd) = stream.next() {
            let new_offset = stream.byte_offset() as u64;
            match cmd? {
                Command::Set{key, value} => {
                    let res = self.index.get(&key);
                    match res {
                        Some((pos, ..)) => {
                            // if the command is in in-memory index, write the command into kvstore.bk
                            if *pos == read_offset {
                                let cmd = Command::Set { key, value };
                                let write_str = serde_json::to_string_pretty(&cmd)?;
                                new_file_writer.write(write_str.as_bytes())?;
                                new_file_writer.flush()?;
                            }
                            // if the command is not in in-memory index, ignore that
                        },
                        None => {}
                    }
                }
                Command::Rm { .. } => {}
            }
            read_offset = new_offset;
        }
        
        // rebuild the index from kvstore.bk
        let new_file = File::open(new_file_path.clone())?;
        let mut new_index = BTreeMap::new();
        self.offset = rebuild_index(&new_file, &mut new_index)?;
        self.index = new_index;
        // delete kvstore.log, rename kvstore.bk as kvstore.log
        remove_file(self.path.clone())?;
        rename(new_file_path.as_path(), self.path.clone().as_path())?;
        let mut file = OpenOptions::new().read(true).write(true).open(self.path.clone())?;
        file.seek(SeekFrom::End(0))?;
        let writer = BufWriter::new(file);
        self.writer = writer;
        Ok(())
    }
}

fn rebuild_index(file: &File, index: &mut BTreeMap<String, (u64, u64)>) -> Result<u64> {
    // deserialize the text in file
    let mut stream = Deserializer::from_reader(file).into_iter::<Command>();
    let mut pos = 0;
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set{key, ..} => {
                index.insert(key, (pos, new_pos - pos));
            }
            Command::Rm { key } => {
                index.remove(&key);
            }
        }
        pos = new_pos;
    }
    Ok(pos)
}