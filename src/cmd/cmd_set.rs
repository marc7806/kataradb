use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::DataType::{Error, SimpleString};
use crate::resp::RESPParser;
use crate::store::Store;

/// see https://redis.io/commands/set/
pub struct SetCommand;

impl Command for SetCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        if args.len() < 2 {
            parser.write_to_stream(stream, Error(String::from("ERR wrong number of arguments for 'set' command")));
            parser.flush_stream(stream);
            return;
        }

        let key = args[0].clone();
        let value = args[1].clone();

        let mut expiration_duration_ms = -1;
        let mut i = 2;

        while i < args.len() {
            let arg = args[i].clone();

            if arg == "EX" {
                if i + 1 >= args.len() {
                    parser.write_to_stream(stream, Error(String::from("ERR syntax error")));
                    parser.flush_stream(stream);
                    return;
                }

                let duration = args[i + 1].clone();
                match duration.parse::<i64>() {
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
            } else {
                parser.write_to_stream(stream, Error(String::from("ERR syntax error")));
                parser.flush_stream(stream);
                return;
            }
        }

        store.put(&key, value, expiration_duration_ms);

        parser.write_to_stream(stream, SimpleString(String::from("OK")));
        parser.flush_stream(stream);
    }
}