use crate::cmd::handler::Command;
use crate::resp::DataType;
use crate::resp::DataType::SimpleString;
use crate::store::Store;

/// see https://redis.io/commands/ping/
pub struct PingCommand;

impl Command for PingCommand {
    fn execute(&self, _: &mut Vec<String>, _: &mut Store) -> DataType {
        return SimpleString(String::from("PONG"));
    }
}