use std::net::TcpStream;

use crate::cmd::handler::Command;
use crate::resp::{DataType, RESPParser};
use crate::store::Store;

/// see: https://redis.io/commands/expire/
pub struct ExpireCommand;

impl Command for ExpireCommand {
    fn execute(&self, args: &mut Vec<String>, parser: &mut RESPParser, stream: &mut TcpStream, store: &mut Store) {
        if args.len() < 3 {
            parser.write_to_stream(stream, DataType::Error(String::from("Wrong number of arguments")));
            parser.flush_stream(stream);
            return;
        }

        let key = &args[1];
        let seconds = &args[2];
        let seconds_int = seconds.parse::<i64>();

        let mut store_object = store.get(key);

        match store_object {
            None => {
                parser.write_to_stream(stream, DataType::Integer(0));
                parser.flush_stream(stream);
                return;
            }
            Some(mut obj) => {
                if seconds_int.is_err() {
                    parser.write_to_stream(stream, DataType::Integer(0));
                    parser.flush_stream(stream);
                    return;
                }

                let seconds_int = seconds_int.unwrap();

                store.put(key, obj.get_data(), seconds_int * 1000);

                parser.write_to_stream(stream, DataType::Integer(1));
                parser.flush_stream(stream);

            }
        }
    }
}