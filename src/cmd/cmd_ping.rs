use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::SimpleString;
use crate::store::Store;

/// see https://redis.io/commands/ping/
pub struct PingCommand;

impl Command for PingCommand {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType {
        return SimpleString(String::from("PONG"));
    }
}