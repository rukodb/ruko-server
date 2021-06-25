use crate::{Ruko, ShardedCollection};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub(crate) struct RequestData {
    id: String,
    command: DataCommand,
}

pub(crate) enum DbCommand {
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

pub(crate) type DbCommandResult = Result<Option<Value>, String>;

pub(crate) enum DataCommand {
    Set { key: Vec<String>, value: Value },
    Get { key: Vec<String> },
    ShutdownShardedCollection,
}

/// returns whether or not it should shut down
pub(crate) fn process_request(request: RequestData) -> bool {
    match request.command {
        DataCommand::Get { key } => {
            println!("Get {} {:?}", request.id, key);
        }
        DataCommand::Set { key, value } => {
            println!("Set {} {:?} {}", request.id, key, value);
        }
        DataCommand::ShutdownShardedCollection => return true,
    };
    false
}

impl Ruko {
    pub(crate) fn run_command(&mut self, cmd: DbCommand) -> DbCommandResult {
        match cmd {
            DbCommand::CreateCollection { name } => {
                let sharded_collection = ShardedCollection::new(self.num_shards);
                self.collections.insert(name, sharded_collection);
                Ok(None)
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
                    Ok(None)
                }
                None => Err("No such collection".to_string()),
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
                        Ok(None)
                    }
                    None => Err("No such collection".to_string()),
                }
            }
        }
    }
}
