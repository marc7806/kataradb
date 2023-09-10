use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::RESPParser;
use crate::resp::DataType::Integer;
use crate::store::Store;

/// see: https://redis.io/commands/del/
pub struct DelCommand;

impl Command for DelCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let mut deleted = 0;

        for i in 1..args.len() {
            let key_to_delete = &args[i];
            let result = store.remove(key_to_delete);
            if result.is_some() {
                deleted += 1;
            }
        }

        parser.write_to_stream(stream, Integer(deleted));
        parser.flush_stream(stream);
    }
}