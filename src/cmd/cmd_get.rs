use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::{Error};
use crate::store::{Store, store_object_to_datatype};

/// see https://redis.io/commands/get/
pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let key = args[0].clone();

        return match store.get(key.as_str()) {
            Some(store_object) => {
                store_object_to_datatype(&store_object)
            }
            None => {
                Error(String::from("Key not found"))
            }
        }
    }
}