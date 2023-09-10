use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::RESPParser;
use crate::resp::DataType::{BulkString, Error};
use crate::store::Store;

/// see https://redis.io/commands/get/
pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let key = args[1].clone();

        match store.get(key.as_str()) {
            Some(store_object) => {
                parser.write_to_stream(stream, BulkString(store_object.data.clone()));
                parser.flush_stream(stream);
            }
            None => {
                parser.write_to_stream(stream, Error(String::from("Key not found")));
                parser.flush_stream(stream);
            }
        }
    }
}