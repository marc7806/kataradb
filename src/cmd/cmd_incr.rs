use crate::cmd::command::Command;
use crate::object_type_encoding::{OBJ_ENCODING_INT, OBJ_TYPE_STRING};
use crate::resp::DataType;
use crate::store::{ObjectValue, Store};

/// see https://redis.io/commands/incr/

pub struct IncrCommand;

impl Command for IncrCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        let key = args[0].clone();

        return match store.get(key.as_str()) {
            Some(store_object) => {
                let value = match store_object.get_value_clone() {
                    ObjectValue::String(string) => {
                        if string.parse::<i64>().is_err() {
                            return DataType::Error(String::from("value is not an integer or out of range"));
                        } else {
                            string.parse::<i64>().unwrap()
                        }
                    }
                };

                let new_value = value + 1;
                store.put(key.as_str(), ObjectValue::String(new_value.to_string()), -1, OBJ_TYPE_STRING | OBJ_ENCODING_INT);
                DataType::Integer(new_value)
            }
            None => {
                store.put(key.as_str(), ObjectValue::String(String::from("1")), -1, OBJ_TYPE_STRING | OBJ_ENCODING_INT);
                DataType::Integer(1)
            }
        };
    }
}
