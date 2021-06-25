use crate::{Ruko, ShardedCollection};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Condvar, Mutex};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

type DataCommandReturnVal = Option<Value>;

pub(crate) struct DataCommandResult {
    value: Arc<(Mutex<Option<DataCommandReturnVal>>, Condvar)>,
}

/// Result of running a DataCommand on an entry of a collection
impl DataCommandResult {
    fn create_pair() -> (DataCommandResult, DataCommandResult) {
        let result = DataCommandResult {
            value: Arc::new((Mutex::new(None), Condvar::new())),
        };
        let result2 = DataCommandResult { value: Arc::clone(&result.value) };
        (result, result2)
    }

    /// Sets command return value and triggers linked result to unblock
    fn set(&mut self, return_val: Option<Value>) {
        let (lock, cvar) = &*self.value;
        let mut maybe_return_val = lock.lock().unwrap();
        *maybe_return_val = Some(return_val);
        cvar.notify_all();
    }

    /// Blocks until the command is done and gives the command return value
    fn get(&mut self) -> DataCommandReturnVal {
        let (lock, cvar) = &*self.value;
        let mut maybe_return_val = lock.lock().unwrap();
        while maybe_return_val.is_none() {
            maybe_return_val = cvar.wait(maybe_return_val).unwrap();
        }
        maybe_return_val.clone().unwrap()
    }
}

pub(crate) struct RequestData {
    id: String,
    command: DataCommand,
    result: DataCommandResult,
}

#[allow(dead_code)]

pub(crate) enum DbCommand {
    CreateCollection {
        name: String,
    },
    DestroyCollection {
        name: String,
    },
    ExecuteOnEntry {
        collection_name: String,
        id: String,
        command: DataCommand,
    },
    ListCollections,
}

pub(crate) type DbCommandResult = Result<Option<Value>, String>;

#[allow(dead_code)]

pub(crate) enum DataCommand {
    Set { key: Vec<String>, value: Value },
    Get { key: Vec<String> },
    ShutdownShardedCollection,
}

/// returns whether or not it should shut down
pub(crate) fn process_request(mut request: RequestData) -> bool {
    match request.command {
        DataCommand::Get { key } => {
            println!("Get {} {:?}", request.id, key);
            request.result.set(Some(Value::String("this is the result".to_owned())));
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
                        let (mut result, result_for_shard) = DataCommandResult::create_pair();
                        shard
                            .request_sender
                            .send(RequestData {
                                id: format!("{}", id),
                                command: DataCommand::ShutdownShardedCollection,
                                result: result_for_shard,
                            })
                            .expect("send shutdown");
                        result.get();
                        shard.join_handle.join().expect("thread shutdown: join");
                    }
                    Ok(None)
                }
                None => Err("No such collection".to_string()),
            },
            DbCommand::ExecuteOnEntry {
                collection_name: name,
                id,
                command,
            } => match self.collections.get(&name) {
                Some(sharded_collection) => {
                    let (mut result, result_for_shard) = DataCommandResult::create_pair();
                    let hash_bucket_size: u64 =
                        std::u64::MAX / TryInto::<u64>::try_into(self.num_shards).expect("convert");
                    let thread_idx: usize = (calculate_hash(&id) / hash_bucket_size)
                        .try_into()
                        .expect("convert");
                    let shard = &sharded_collection.shards[thread_idx];
                    shard
                        .request_sender
                        .send(RequestData { id, command, result: result_for_shard })
                        .expect("send modify command");
                    Ok(result.get())
                }
                None => Err("No such collection".to_string()),
            },
            DbCommand::ListCollections => Ok(Some(Value::Array(
                self.collections
                    .keys()
                    .map(|x| Value::String(x.clone()))
                    .collect(),
            ))),
        }
    }
}
