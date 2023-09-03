use std::net::TcpStream;

use crate::resp::{DataType, RESPParser};
use crate::resp::DataType::{BulkString, SimpleString};

pub fn handle_cmd(parser: &mut RESPParser, data_type: DataType, stream: &mut TcpStream) {
    match data_type {
        DataType::Array(array) => {
            let cmd = &array[0];

            if cmd == &BulkString(String::from("COMMAND")) {} else if cmd == &BulkString(String::from("PING")) {
                parser.write_to_stream(stream, SimpleString(String::from("PONG")));
                parser.flush_stream(stream);

                while let Ok(data_type) = parser.decode_next(stream) {
                    handle_cmd(parser, data_type, stream);
                }
            } else {}
        }
        _ => {
            println!("Got not supported command");
        }
    }
}