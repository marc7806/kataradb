use std::collections::HashMap;
use std::str::FromStr;
use crate::cmd::cmd_bgrewriteaof::BgRewriteAofCommand;
use crate::cmd::cmd_del::DelCommand;
use crate::cmd::cmd_expire::ExpireCommand;
use crate::cmd::cmd_get::GetCommand;
use crate::cmd::cmd_incr::IncrCommand;
use crate::cmd::cmd_info::InfoCommand;
use crate::cmd::cmd_ping::PingCommand;
use crate::cmd::cmd_set::SetCommand;
use crate::cmd::cmd_ttl::TTLCommand;
use crate::cmd::command::SimpleCommand::{BGREWRITEAOF, DEL, EXPIRE, GET, INCR, INFO, PING, SET, TTL};
use crate::resp::DataType;
use crate::store::Store;

pub trait Command {
    fn execute(&self, args: &mut Vec<String>, store: &mut Store) -> DataType;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SimpleCommand {
    PING,
    SET,
    GET,
    TTL,
    DEL,
    EXPIRE,
    BGREWRITEAOF,
    INCR,
    INFO,
}

impl FromStr for SimpleCommand {
    type Err = ();
    fn from_str(input: &str) -> Result<SimpleCommand, Self::Err> {
        match input {
            "PING" => Ok(PING),
            "SET" => Ok(SET),
            "GET" => Ok(GET),
            "TTL" => Ok(TTL),
            "DEL" => Ok(DEL),
            "EXPIRE" => Ok(EXPIRE),
            "BGREWRITEAOF" => Ok(BGREWRITEAOF),
            "INCR" => Ok(INCR),
            "INFO" => Ok(INFO),
            _ => Err(()),
        }
    }
}

pub fn is_simple_command(cmd: &DataType) -> Option<SimpleCommand> {
    return match cmd {
        DataType::SimpleString(value) => {
            value.parse::<SimpleCommand>().ok()
        }
        DataType::BulkString(value) => {
            value.parse::<SimpleCommand>().ok()
        }
        _ => {
            None
        }
    }
}

pub fn get_commands() -> HashMap<SimpleCommand, Box<dyn Command>> {
    let mut commands: HashMap<SimpleCommand, Box<dyn Command>> = HashMap::new();

    commands.insert(PING, Box::new(PingCommand));
    commands.insert(SET, Box::new(SetCommand));
    commands.insert(GET, Box::new(GetCommand));
    commands.insert(TTL, Box::new(TTLCommand));
    commands.insert(DEL, Box::new(DelCommand));
    commands.insert(EXPIRE, Box::new(ExpireCommand));
    commands.insert(BGREWRITEAOF, Box::new(BgRewriteAofCommand));
    commands.insert(INCR, Box::new(IncrCommand));
    commands.insert(INFO, Box::new(InfoCommand));

    return commands;
}
