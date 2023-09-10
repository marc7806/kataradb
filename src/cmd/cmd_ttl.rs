use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, Error, Integer};
use crate::store::Store;

/// see https://redis.io/commands/ttl/
pub struct TTLCommand;

impl Command for TTLCommand {
    fn execute(&self, data: &mut Vec<DataType>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        let key = data[1].clone();

        match key {
            BulkString(key_str) => {
                match store.get(&key_str) {
                    Some(store_object) => {
                        let now = chrono::Utc::now().timestamp_millis();
                        let expires_at = store_object.expires_at;

                        if expires_at == -1 {
                            parser.write_to_stream(stream, Integer(-1));
                            parser.flush_stream(stream);
                        } else {
                            let ttl = expires_at - now;
                            let ttl_seconds = ttl / 1000;
                            parser.write_to_stream(stream, Integer(ttl_seconds));
                            parser.flush_stream(stream);
                        }
                    }
                    None => {
                        parser.write_to_stream(stream, Integer(-2));
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