use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::store::Store;

/// see: https://redis.io/commands/expire/
pub struct ExpireCommand;

impl Command for ExpireCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        if args.len() < 2 {
            return DataType::Error(String::from("Wrong number of arguments"));
        }

        let key = &args[0];
        let seconds = &args[1];
        let seconds_int = seconds.parse::<i64>();

        let store_object = store.get(key);

        return match store_object {
            None => {
                return DataType::Integer(0);
            }
            Some(mut obj) => {
                if seconds_int.is_err() {
                    return DataType::Integer(0);
                }

                let seconds_int = seconds_int.unwrap();
                store.put(key, obj.get_value_clone(), seconds_int * 1000, obj.type_encoding);
                return DataType::Integer(1);
            }
        }
    }
}