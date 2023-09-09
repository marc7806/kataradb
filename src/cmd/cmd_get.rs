use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error};
use crate::store::Store;

pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let key = data[1].clone();

        match key {
            BulkString(key_str) => {
                match store.get::<DataType>(key_str.as_str()) {
                    Ok(value) => {
                        parser.write_to_stream(stream, value.clone());
                        parser.flush_stream(stream);
                    }
                    Err(_) => {
                        parser.write_to_stream(stream, BulkString(String::from("")));
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