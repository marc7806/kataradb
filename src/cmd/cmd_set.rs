use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error, SimpleString};
use crate::store::Store;

/// see https://redis.io/commands/set/
pub struct SetCommand;

impl Command for SetCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        if data.len() < 3 {
            parser.write_to_stream(stream, Error(String::from("ERR wrong number of arguments for 'set' command")));
            parser.flush_stream(stream);
            return;
        }

        let key = data[1].clone();
        let value = data[2].clone();

        let mut expiration_duration_ms = -1;
        let mut i = 3;

        while i < data.len() {
            let arg = data[i].clone();

            if arg == BulkString(String::from("EX")) {
                if i + 1 >= data.len() {
                    parser.write_to_stream(stream, Error(String::from("ERR syntax error")));
                    parser.flush_stream(stream);
                    return;
                }

                let duration = data[i + 1].clone();
                match duration {
                    BulkString(duration_str) => {
                        match duration_str.parse::<i64>() {
                            Ok(duration_sec) => {
                                expiration_duration_ms = duration_sec * 1000;
                                i += 2;
                            }
                            Err(_) => {
                                parser.write_to_stream(stream, Error(String::from("ERR value is not an integer or out of range")));
                                parser.flush_stream(stream);
                                return;
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                parser.write_to_stream(stream, Error(String::from("ERR syntax error")));
                parser.flush_stream(stream);
                return;
            }
        }

        match key {
            BulkString(key_str) => {
                store.put(&key_str, value, expiration_duration_ms);

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