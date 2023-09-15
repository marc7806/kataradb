use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::{BulkString, Error};
use crate::store::Store;

/// see https://redis.io/commands/get/
pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let key = args[0].clone();

        return match store.get(key.as_str()) {
            Some(store_object) => {
                BulkString(store_object.data.clone())
            }
            None => {
                Error(String::from("Key not found"))
            }
        }
    }
}