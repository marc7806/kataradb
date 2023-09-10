use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::DataType::Integer;
use crate::resp::RESPParser;
use crate::store::Store;

/// see: https://redis.io/commands/del/
pub struct DelCommand;

impl Command for DelCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let mut deleted = 0;

        for key_to_delete in args.iter() {
            let result = store.remove(key_to_delete);
            if result.is_some() {
                deleted += 1;
            }
        }

        parser.write_to_stream(stream, Integer(deleted));
        parser.flush_stream(stream);
    }
}