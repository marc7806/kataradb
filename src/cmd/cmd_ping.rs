use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::RESPParser;
use crate::resp::DataType::SimpleString;
use crate::store::Store;

/// see https://redis.io/commands/ping/
pub struct PingCommand;

impl Command for PingCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        parser.write_to_stream(stream, SimpleString(String::from("PONG")));
        parser.flush_stream(stream);
    }
}