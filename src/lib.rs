//! Ruko in-memory sharded kv database.
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

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

impl Ruko {
    pub fn new() -> Ruko {
        Ruko {
            num_shards: num_cpus::get(),
            collections: HashMap::new(),
        }
    }
}

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
    Success { result: Option<Value> },
    Error { message: String },
}

enum DataCommand {
    Set { key: Vec<String>, value: Value },
    Get { key: Vec<String> },
    ShutdownShardedCollection,
}

fn spawn_shard_worker(recv: Receiver<RequestData>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match recv.recv() {
            Ok(request) => match request.command {
                DataCommand::Get { key } => {
                    println!("Get {} {:?}", request.id, key);
                }
                DataCommand::Set { key, value } => {
                    println!("Set {} {:?} {}", request.id, key, value);
                }
                DataCommand::ShutdownShardedCollection => break,
            },
            Err(err) => {
                println!("Worker {:?} receive error: {}", thread::current().id(), err);
                break;
            }
        }
    })
}

impl Ruko {
    fn run_command(&mut self, cmd: DbCommand) -> DbCommandResult {
        match cmd {
            DbCommand::CreateCollection { name } => {
                let sharded_collection = ShardedCollection::new(self.num_shards);
                self.collections.insert(name, sharded_collection);
                DbCommandResult::Success { result: None }
            }
            DbCommand::DestroyCollection { name } => match self.collections.remove(&name) {
                Some(collection) => {
                    for (id, shard) in collection.shards.into_iter().enumerate() {
                        shard
                            .request_sender
                            .send(RequestData {
                                id: format!("{}", id),
                                command: DataCommand::ShutdownShardedCollection,
                            })
                            .expect("send shutdown");
                        shard.join_handle.join().expect("thread shutdown: join");
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
                        let hash_bucket_size: u64 = std::u64::MAX
                            / TryInto::<u64>::try_into(self.num_shards).expect("convert");
                        let thread_idx: usize = (calculate_hash(&id) / hash_bucket_size)
                            .try_into()
                            .expect("convert");
                        let shard = &sharded_collection.shards[thread_idx];
                        shard
                            .request_sender
                            .send(RequestData { id, command })
                            .expect("send modify command");
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
        // clone our keys
        let keys: Vec<String> = self.collections.keys().map(|k| k.clone()).collect();
        // go over all keys, calling DestroyCollection
        for key in keys {
            self.run_command(DbCommand::DestroyCollection { name: key });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn easy() {
        let mut db = Ruko::new();
        let cmd = DbCommand::CreateCollection {
            name: String::from("test_collection"),
        };
        db.run_command(cmd);
        let cmd = DbCommand::ModifyCollection {
            id: String::from("test_collection"),
            name: String::from("test_collection"),
            command: DataCommand::Set {
                key: vec![String::from("key1")],
                value: json!({"zach": 9000}),
            },
        };
        db.run_command(cmd);
        db.shutdown();
        assert_eq!(2 + 2, 4);
    }
}
