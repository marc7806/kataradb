use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error, Integer};
use crate::store::Store;

/// see: https://redis.io/commands/del/
pub struct DelCommand;

impl Command for DelCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let mut deleted = 0;

        for i in 1..data.len() {
            match &data[i] {
                BulkString(key_str) => {
                    let result = store.remove(key_str);
                    if result.is_some() {
                        deleted += 1;
                    }
                }
                _ => {
                    parser.write_to_stream(stream, Error(String::from("Key must be a bulk string")));
                    parser.flush_stream(stream);
                    return;
                }
            }
        }

        parser.write_to_stream(stream, Integer(deleted));
        parser.flush_stream(stream);
    }
}