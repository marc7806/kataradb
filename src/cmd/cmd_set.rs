use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error, SimpleString};
use crate::store::Store;

pub struct SetCommand;

impl Command for SetCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let key = data[1].clone();
        let value = data[2].clone();

        match key {
            BulkString(key_str) => {
                store.put(key_str.as_str(), value);

                parser.write_to_stream(stream, SimpleString(String::from("OK")));
                parser.flush_stream(stream);
            }
            _ => {
                parser.write_to_stream(stream, Error(String::from("Key must be a bulk string")));
                parser.flush_stream(stream);
                return;
            }
        }
    }
}