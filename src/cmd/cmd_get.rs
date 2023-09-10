use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error};
use crate::store::Store;

/// see https://redis.io/commands/get/
pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let key = data[1].clone();

        match key {
            BulkString(key_str) => {
                match store.get(&key_str) {
                    Some(store_object) => {
                        parser.write_to_stream(stream, store_object.data.clone());
                        parser.flush_stream(stream);
                    }
                    None => {
                        parser.write_to_stream(stream, Error(String::from("Key not found")));
                        parser.flush_stream(stream);
                    }
                }
            }
            _ => {
                parser.write_to_stream(stream, Error(String::from("Key must be a bulk string")));
                parser.flush_stream(stream);
                return;
            }
        }
    }
}