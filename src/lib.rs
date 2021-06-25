//! Ruko in-memory sharded kv database.
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

mod cmd;
use cmd::process_request;
use cmd::{DbCommand, RequestData};

#[derive(Debug)]
pub struct Shard {
    request_sender: Sender<RequestData>,
    join_handle: JoinHandle<()>,
}

#[derive(Debug)]
pub struct ShardedCollection {
    shards: Vec<Shard>,
}

impl ShardedCollection {
    fn new(num_shards: usize) -> ShardedCollection {
        let mut shards = vec![];
        for _ in 0..num_shards {
            let (request_sender, request_receiver) = channel::<RequestData>();
            let join_handle = spawn_shard_worker(request_receiver);
            shards.push(Shard {
                request_sender,
                join_handle,
            });
        }

        ShardedCollection { shards }
    }
}

#[derive(Debug)]
pub struct Ruko {
    num_shards: usize,
    collections: HashMap<String, ShardedCollection>,
}

impl Default for Ruko {
    fn default() -> Self {
        Ruko::new()
    }
}

impl Ruko {
    pub fn new() -> Ruko {
        Ruko {
            num_shards: num_cpus::get(),
            collections: HashMap::new(),
        }
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        // clone our keys
        let keys: Vec<String> = self.collections.keys().cloned().collect();
        // go over all keys, calling DestroyCollection
        for key in keys {
            self.run_command(DbCommand::DestroyCollection { name: key })?;
        }
        Ok(())
    }
}

fn spawn_shard_worker(recv: Receiver<RequestData>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match recv.recv() {
            Ok(request) => {
                if process_request(request) {
                    break;
                }
            }
            Err(err) => {
                println!("Worker {:?} receive error: {}", thread::current().id(), err);
                break;
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::DataCommand;

    use serde_json::json;
    #[test]
    fn easy() -> Result<(), String> {
        let mut db = Ruko::new();
        let cmd = DbCommand::CreateCollection {
            name: String::from("test_collection"),
        };
        db.run_command(cmd)?;
        let cmd = DbCommand::ModifyCollection {
            id: String::from("test_collection"),
            name: String::from("test_collection"),
            command: DataCommand::Set {
                key: vec![String::from("key1")],
                value: json!({"zach": 9000}),
            },
        };
        db.run_command(cmd)?;
        db.shutdown()
    }
}
