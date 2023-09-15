use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::{Error, SimpleString};
use crate::store::Store;

/// see https://redis.io/commands/set/
pub struct SetCommand;

impl Command for SetCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        if args.len() < 2 {
            return Error(String::from("ERR wrong number of arguments for 'set' command"));
        }

        let key = args[0].clone();
        let value = args[1].clone();

        let mut expiration_duration_ms = -1;
        let mut i = 2;

        while i < args.len() {
            let arg = args[i].clone();

            if arg == "EX" {
                if i + 1 >= args.len() {
                    return Error(String::from("ERR syntax error"));
                }

                let duration = args[i + 1].clone();
                match duration.parse::<i64>() {
                    Ok(duration_sec) => {
                        expiration_duration_ms = duration_sec * 1000;
                        i += 2;
                    }
                    Err(_) => {
                        return Error(String::from("ERR value is not an integer or out of range"));
                    }
                }
            } else {
                return Error(String::from("ERR syntax error"));
            }
        }

        store.put(&key, value, expiration_duration_ms);
        return SimpleString(String::from("OK"));
    }
}