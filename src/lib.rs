//! Ruko in-memory sharded kv database.
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::thread::Thread;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

struct RequestData {
    id: String,
    command: DataCommand,
}

#[derive(Debug)]
pub struct Shard {
    work_sender: Sender<RequestData>,
    join_handle: JoinHandle<()>,
}

#[derive(Debug)]
pub struct ShardedCollection {
    shards: Vec<Shard>,
}

impl ShardedCollection {
    fn new() -> ShardedCollection {
        ShardedCollection { shards: Vec::new() }
    }
}

#[derive(Debug)]
pub struct Ruko {
    num_cores: usize,
    collections: HashMap<String, ShardedCollection>,
}

impl Ruko {
    fn new() -> Ruko {
        Ruko {
            num_cores: num_cpus::get(),
            collections: HashMap::new(),
        }
    }
}

pub struct Worker {
    id: usize,
}

// Placeholders
type JNode = String;
enum DbCommand {
    CreateCollection {
        name: String,
    },
    DestroyCollection {
        name: String,
    },
    ModifyCollection {
        name: String,
        id: String,
        command: DataCommand,
    },
}

enum DbCommandResult {
    Success { result: Option<JNode> },
    Error { message: String },
}

enum DataCommand {
    Set { key: Vec<String>, value: JNode },
    Get { key: Vec<String> },
    ShutdownShardedCollection,
}

pub fn mainThread() {
    let db: Ruko;
}

fn collection_shard_worker(work_receiver: Receiver<RequestData>) {
    // let data: HashMap<String, Value>;
    loop {
        match work_receiver.recv() {
            Ok(request) => match request.command {
                DataCommand::Get { key } => {
                    println!("Get {} {:?}", request.id, key);
                }
                DataCommand::Set { key, value } => {
                    println!("Set {} {:?} {}", request.id, key, value);
                }
                ShutdownShardedCollection => break,
            },
            Err(err) => {
                println!("Err {}", err);
                break;
            }
        }
    }
}

impl Ruko {
    fn runDbCommand(&mut self, db_command: DbCommand) -> DbCommandResult {
        match db_command {
            DbCommand::CreateCollection { name } => {
                let mut sharded_collection = ShardedCollection::new();
                for _ in 0..self.num_cores {
                    let (work_sender, work_receiver) = channel::<RequestData>();
                    let join_handle = thread::spawn(|| collection_shard_worker(work_receiver));
                    sharded_collection.shards.push(Shard {
                        work_sender,
                        join_handle,
                    });
                }
                self.collections.insert(name, sharded_collection);
                DbCommandResult::Success { result: None }
            }
            DbCommand::DestroyCollection { name } => match self.collections.remove(&name) {
                Some(collection) => {
                    for shard in collection.shards {
                        shard
                            .work_sender
                            .send(RequestData {
                                id: String::new(),
                                command: DataCommand::ShutdownShardedCollection,
                            })
                            .unwrap();
                        shard.join_handle.join();
                    }
                    DbCommandResult::Success { result: None }
                }
                None => DbCommandResult::Error {
                    message: "No such collection".to_string(),
                },
            },
            DbCommand::ModifyCollection { name, id, command } => {
                match self.collections.get(&name) {
                    Some(sharded_collection) => {
                        let hash_bucket_size = std::u64::MAX / (self.num_cores as u64);
                        let thread_idx = (calculate_hash(&id) / hash_bucket_size) as usize;
                        let shard = &sharded_collection.shards[thread_idx];
                        shard
                            .work_sender
                            .send(RequestData {
                                id: id,
                                command: command,
                            })
                            .unwrap();
                        DbCommandResult::Success { result: None }
                    }
                    None => DbCommandResult::Error {
                        message: "No such collection".to_string(),
                    },
                }
            }
        }
    }

    fn shutdown(&mut self) {
        for key in self.collections.keys().collect::<Vec<&String>>() {
            let k2 = (*key).clone();
            (&mut self).runDbCommand(DbCommand::DestroyCollection { name: k2 });
        }
    }
}
