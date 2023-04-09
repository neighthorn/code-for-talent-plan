use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter, Write, Seek, SeekFrom, Read};
use std::path::PathBuf;
use std::fs::{File, OpenOptions, remove_file, rename};
use std::sync::{Arc, Mutex};

use crossbeam_skiplist::SkipMap;
use log::info;
use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

use crate::{KvStoreError, Result, KvEngine};

const COMPACT_THRESHOLD: u64 = 1024 * 1024;

/// 'Command' is a enum that represents various commands
#[derive(Serialize, Deserialize)]
pub enum Command {
    Set{key: String, value: String},
    Rm{key: String},
}

/// 'KvStore' is a Hashmap that can store key-value pairs
#[derive(Clone)]
pub struct KvStore {
    // in-memory index, stores the log pointer(file offset and command length) of the command
    index: Arc<SkipMap<String, (u64, u64)>>,
    // writer buffer for log
    writer: Arc<Mutex<BufWriter<File>>>,
    // the path of the file
    path: Arc<PathBuf>,
    // the dir of the file
    dir: Arc<PathBuf>,
    // the write offset of the log file
    offset: Arc<Mutex<u64>>,
}

impl KvStore {
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
        let mut index= SkipMap::new();
        let offset = rebuild_index(&file, &mut index)?;
        file.seek(SeekFrom::End(0))?;
        let writer = BufWriter::new(file);
        Ok(KvStore { 
            index: Arc::new((index)), 
            writer: Arc::new(Mutex::new(writer)), 
            path: Arc::new(path), 
            dir: Arc::new(dir), 
            offset: Arc::new(Mutex::new(offset))
        })
    }

    /// used to compact the kvstore and the log, remove the redundant key-value command
    pub fn compact(&self) -> Result<()> {
        let mut new_file_path = self.dir.clone();
        let mut new_file_path = (*new_file_path).clone();
        new_file_path.push("kvstore.bk");
        // kvstore.bk
        let new_file = OpenOptions::new()
                    .read(true).write(true).create(true).open(new_file_path.clone())?;
        // kvstore.log
        let path = self.path.clone();
        let old_file = File::open((*path).clone())?;
        let mut stream = Deserializer::from_reader(old_file).into_iter::<Command>();
        let mut new_file_writer = BufWriter::new(new_file);

        // read every command in log file
        let mut read_offset: u64 = 0;
        while let Some(cmd) = stream.next() {
            let new_offset = stream.byte_offset() as u64;
            match cmd? {
                Command::Set{key, value} => {
                    let mut index = self.index.clone();
                    let res = index.get(&key);
                    match res {
                        Some(kv) => {
                            let (pos, _) = kv.value();
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
        let mut index = self.index.clone();
        index.clear();
        let offset = self.offset.clone();
        let mut offset = offset.lock().unwrap();
        // *offset = rebuild_index(&new_file, &mut index)?;
        // 这里不能调用rebuild_index的原因是不能把index作为可变借用，但是为什么呢，感觉理论上可以的
        let mut pos = 0;
        let mut stream = Deserializer::from_reader(new_file).into_iter::<Command>();
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
        *offset = pos;
        
        // delete kvstore.log, rename kvstore.bk as kvstore.log
        let path = self.path.clone();
        remove_file((*path).clone())?;
        rename(new_file_path.as_path(), path.as_path())?;
        let writer = self.writer.clone();
        let mut writer = writer.lock().unwrap();
        let mut file = OpenOptions::new().read(true).write(true).open((*path).clone())?;
        file.seek(SeekFrom::End(0))?;
        // 这里可以直接对writer进行修改而不能对index进行修改，主要是因为BufWriter有deref trait，而SkipMap没有deref trait
        *writer = BufWriter::new(file);
        Ok(())
    }
}

fn rebuild_index(file: &File, index: &mut SkipMap<String, (u64, u64)>) -> Result<u64> {
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


impl KvEngine for KvStore {
    /// insert a key-value pair in KvStore
    fn set(&self, key: String, value: String) -> Result<()> {
        // construct set command
        let cmd = Command::Set{ key, value };
        // serialize the command to a string
        let write_str = serde_json::to_string_pretty(&cmd)?;
        // write command into the disk
        // 问题：这里为什么不能写成self.writer.clone().lock().unwrap()呢
        let writer = self.writer.clone();
        let mut writer = writer.lock().unwrap();
        let len = writer.write(write_str.as_bytes())?;
        writer.flush()?;
        // take the command offset in the file as value
        let offset = self.offset.clone();
        let mut offset = offset.lock().unwrap();
        let cmd_offset = *offset;
        // update the end offset of the file
        *offset = *offset + len as u64;
        // update the in-memory index
        let mut index = self.index.clone();
        if let Command::Set { key, .. } = cmd {
            index.insert(key, (cmd_offset, len as u64));
        }
        drop(writer);
        if *offset > COMPACT_THRESHOLD {
            drop(offset);
            self.compact()?;
        }
        Ok(())
    }
    /// get the value for the given key
    fn get(&self, key: String) -> Result<Option<String>> {
        // 这根本就没并行起来，这个index在读的时候不能也用Mutex来锁，看看要怎么改呢
        // get the offset of the latest command corresponds to key
        let mut index = self.index.clone();
        let res = index.get(&key);
        match res {
            Some(kv) => {
                let (pos, len) = kv.value();
                // make the pointer of the file to the command offset
                let path = self.path.clone();
                let mut file = File::open((*path).clone())?;
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
        // Ok(None)
    }
    /// reomve the key-value pair with given key
    fn remove(&self, key: String) -> Result<()> {
        let mut index = self.index.clone();
        if index.contains_key(&key) {
            // construct remove command
            let cmd = Command::Rm { key };
            // serialize the command to a string
            let write_str = serde_json::to_string_pretty(&cmd)?;
            // write command into the disk
            let writer = self.writer.clone();
            let mut writer = writer.lock().unwrap();
            let len = writer.write(write_str.as_bytes())?;
            writer.flush()?;
            // update the end offset of the file
            let offset = self.offset.clone();
            let mut offset = offset.lock().unwrap();
            *offset += len as u64;
            // update the in-memory index
            if let Command::Rm { key } = cmd {
                index.remove(&key);
            }
            if *offset > COMPACT_THRESHOLD {
                self.compact()?;
            }
            return Ok(());
        } else {
            return Err(KvStoreError::RemoveNonExistKey);
        }
        Ok(())
   }
}